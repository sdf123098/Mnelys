//! Starting, watching and stopping Minecraft processes.
//!
//! Each launch gets a session record the UI can poll. Output is captured into a bounded
//! ring buffer so the task center can show the last lines of a crashed launch without
//! keeping an unbounded log in memory.

use std::collections::{HashMap, VecDeque};
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use mnelys_application::ports::{Clock, GameRunner};
use mnelys_domain::{CoreError, CoreResult, LaunchPlan, LaunchSession, LaunchState};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::Mutex as AsyncMutex;

const LOG_CAPACITY: usize = 4000;
const POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(250);
const MAX_SESSIONS: usize = 16;

struct SessionSlot {
    session: Mutex<LaunchSession>,
    child: AsyncMutex<tokio::process::Child>,
    stop_requested: AtomicBool,
    logs: Mutex<VecDeque<String>>,
}

impl SessionSlot {
    fn snapshot(&self) -> LaunchSession {
        self.session
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    fn push_log(&self, line: String) {
        let mut logs = self
            .logs
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if logs.len() == LOG_CAPACITY {
            logs.pop_front();
        }
        logs.push_back(line);
    }
}

/// The real [`GameRunner`].
pub struct TokioGameRunner {
    clock: Arc<dyn Clock>,
    sessions: Mutex<HashMap<String, Arc<SessionSlot>>>,
}

impl TokioGameRunner {
    #[must_use]
    pub fn new(clock: Arc<dyn Clock>) -> Self {
        Self {
            clock,
            sessions: Mutex::new(HashMap::new()),
        }
    }

    fn slot(&self, session_id: &str) -> CoreResult<Arc<SessionSlot>> {
        self.sessions
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(session_id)
            .cloned()
            .ok_or_else(|| CoreError::unavailable(format!("session `{session_id}` does not exist")))
    }

    fn prune(&self) {
        let mut sessions = self
            .sessions
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if sessions.len() < MAX_SESSIONS {
            return;
        }
        let finished: Vec<String> = sessions
            .iter()
            .filter(|(_, slot)| !slot.snapshot().state.is_running())
            .map(|(id, _)| id.clone())
            .collect();
        for id in finished {
            sessions.remove(&id);
        }
    }
}

#[async_trait]
impl GameRunner for TokioGameRunner {
    async fn launch(&self, plan: LaunchPlan) -> CoreResult<LaunchSession> {
        if plan.java_path.trim().is_empty() {
            return Err(CoreError::invalid_request(
                "the launch plan has no Java path",
            ));
        }
        let command_line = plan.command_line();
        let program = command_line
            .first()
            .cloned()
            .ok_or_else(|| CoreError::invalid_request("the launch plan is empty"))?;

        let game_directory = std::path::PathBuf::from(&plan.game_directory);
        tokio::fs::create_dir_all(&game_directory)
            .await
            .map_err(|error| {
                CoreError::io("could not create the game directory").with_details(error.to_string())
            })?;

        let mut command = Command::new(&program);
        command
            .args(&command_line[1..])
            .current_dir(&game_directory)
            .envs(&plan.environment)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = command.spawn().map_err(|error| {
            CoreError::io(format!("could not start {}", plan.java_path))
                .with_details(error.to_string())
        })?;
        let pid = child.id().unwrap_or(0);
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        let session = LaunchSession {
            id: uuid::Uuid::new_v4().to_string(),
            instance_id: plan.instance_id.clone(),
            version_id: plan.version_id.clone(),
            pid,
            state: LaunchState::Running,
            started_at_ms: self.clock.now_ms(),
            finished_at_ms: None,
            exit_code: None,
        };

        let slot = Arc::new(SessionSlot {
            session: Mutex::new(session.clone()),
            child: AsyncMutex::new(child),
            stop_requested: AtomicBool::new(false),
            logs: Mutex::new(VecDeque::new()),
        });

        if let Some(stdout) = stdout {
            capture(stdout, Arc::clone(&slot));
        }
        if let Some(stderr) = stderr {
            capture(stderr, Arc::clone(&slot));
        }

        self.prune();
        self.sessions
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(session.id.clone(), Arc::clone(&slot));

        self.watch(Arc::clone(&slot), Arc::clone(&self.clock));
        Ok(session)
    }

    async fn stop(&self, session_id: &str) -> CoreResult<bool> {
        let slot = self.slot(session_id)?;
        if !slot.snapshot().state.is_running() {
            return Ok(false);
        }
        slot.stop_requested.store(true, Ordering::SeqCst);
        let mut child = slot.child.lock().await;
        child.start_kill().map_err(|error| {
            CoreError::io("the game process could not be stopped").with_details(error.to_string())
        })?;
        Ok(true)
    }

    async fn sessions(&self) -> CoreResult<Vec<LaunchSession>> {
        let mut sessions: Vec<LaunchSession> = self
            .sessions
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .values()
            .map(|slot| slot.snapshot())
            .collect();
        sessions.sort_by_key(|session| std::cmp::Reverse(session.started_at_ms));
        Ok(sessions)
    }

    async fn logs(&self, session_id: &str, limit: usize) -> CoreResult<Vec<String>> {
        let slot = self.slot(session_id)?;
        let logs = slot
            .logs
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let limit = limit.clamp(1, LOG_CAPACITY);
        let skip = logs.len().saturating_sub(limit);
        Ok(logs.iter().skip(skip).cloned().collect())
    }
}

impl TokioGameRunner {
    /// Polls the child until it exits, then records how it ended.
    ///
    /// `try_wait` is used instead of `wait` so [`TokioGameRunner::stop`] can still lock
    /// the child while the monitor is running.
    fn watch(&self, slot: Arc<SessionSlot>, clock: Arc<dyn Clock>) {
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(POLL_INTERVAL).await;
                let status = {
                    let mut child = slot.child.lock().await;
                    match child.try_wait() {
                        Ok(status) => status,
                        Err(error) => {
                            let mut session = slot
                                .session
                                .lock()
                                .unwrap_or_else(std::sync::PoisonError::into_inner);
                            session.state = LaunchState::Failed;
                            session.finished_at_ms = Some(clock.now_ms());
                            slot.push_log(format!(
                                "[Mnelys] could not read the process state: {error}"
                            ));
                            return;
                        }
                    }
                };
                if let Some(status) = status {
                    let mut session = slot
                        .session
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    session.state = if slot.stop_requested.load(Ordering::SeqCst) {
                        LaunchState::Stopped
                    } else {
                        LaunchState::Exited
                    };
                    session.finished_at_ms = Some(clock.now_ms());
                    session.exit_code = status.code();
                    return;
                }
            }
        });
    }
}

fn capture<R>(reader: R, slot: Arc<SessionSlot>)
where
    R: tokio::io::AsyncRead + Unpin + Send + 'static,
{
    tokio::spawn(async move {
        let mut lines = BufReader::new(reader).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            slot.push_log(line);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use mnelys_application::ports::SystemClock;
    use std::collections::BTreeMap;

    fn plan(main_class: &str) -> LaunchPlan {
        LaunchPlan {
            instance_id: String::from("main"),
            version_id: String::from("1.21.4"),
            java_path: String::from("definitely-not-java-mnelys"),
            main_class: main_class.to_owned(),
            classpath: Vec::new(),
            native_path: None,
            game_directory: std::env::temp_dir().to_string_lossy().into_owned(),
            assets_directory: String::from("assets"),
            asset_index: String::from("17"),
            jvm_arguments: Vec::new(),
            game_arguments: Vec::new(),
            environment: BTreeMap::new(),
            memory_mb: 0,
        }
    }

    #[tokio::test]
    async fn an_unstartable_plan_is_reported_as_io() {
        let runner = TokioGameRunner::new(Arc::new(SystemClock));
        let error = runner
            .launch(plan("net.minecraft.client.main.Main"))
            .await
            .expect_err("missing java");
        assert_eq!(error.code, mnelys_domain::CoreErrorCode::Io);
    }

    #[tokio::test]
    async fn an_unknown_session_is_unavailable() {
        let runner = TokioGameRunner::new(Arc::new(SystemClock));
        let error = runner
            .logs("missing", 10)
            .await
            .expect_err("unknown session");
        assert_eq!(error.code, mnelys_domain::CoreErrorCode::Unavailable);
    }
}
