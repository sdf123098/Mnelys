import { getCurrentWindow } from '@tauri-apps/api/window';

/**
 * Native window chrome. Kept beside the command client because this is the only other
 * Tauri surface the webview is allowed to touch (ADR 0005): the custom title bar needs
 * window controls, and nothing else in the UI should reach the window API directly.
 */
export function isDesktopRuntime(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

/**
 * Best-effort platform from the webview user agent, used before `runtime_info` arrives.
 * Only layout depends on it; no security decision may.
 */
export function detectPlatform(): string {
  if (typeof navigator === 'undefined') {
    return 'unknown';
  }
  const agent = navigator.userAgent;
  if (agent.includes('Mac OS X') || agent.includes('Macintosh')) {
    return 'macos';
  }
  if (agent.includes('Linux')) {
    return 'linux';
  }
  if (agent.includes('Windows')) {
    return 'windows';
  }
  return 'unknown';
}

export async function setWindowTitle(title: string): Promise<void> {
  if (!isDesktopRuntime()) {
    return;
  }
  await getCurrentWindow().setTitle(title);
}

export async function minimizeWindow(): Promise<void> {
  if (!isDesktopRuntime()) {
    return;
  }
  await getCurrentWindow().minimize();
}

export async function toggleMaximizeWindow(): Promise<void> {
  if (!isDesktopRuntime()) {
    return;
  }
  await getCurrentWindow().toggleMaximize();
}

export async function closeWindow(): Promise<void> {
  if (!isDesktopRuntime()) {
    return;
  }
  await getCurrentWindow().close();
}

export async function isWindowMaximized(): Promise<boolean> {
  if (!isDesktopRuntime()) {
    return false;
  }
  return getCurrentWindow().isMaximized();
}

/** Subscribes to size changes so the maximize/restore glyph follows the real state. */
export async function onWindowResized(handler: () => void): Promise<() => void> {
  if (!isDesktopRuntime()) {
    return () => undefined;
  }
  return getCurrentWindow().onResized(() => {
    handler();
  });
}
