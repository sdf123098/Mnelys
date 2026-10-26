//! The reqwest-backed HTTP client.
//!
//! One client is shared for the whole process so connections and the TLS session cache
//! are reused. Every request carries the Mnelys user agent, which the Mojang, Modrinth
//! and CurseForge APIs use to identify well-behaved clients.

use std::time::Duration;

use async_trait::async_trait;
use mnelys_application::ports::{HttpBody, HttpClient, HttpMethod, HttpRequest, HttpResponse};
use mnelys_domain::{CoreError, CoreResult};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

/// Identifies Mnelys to every service it talks to.
pub const USER_AGENT: &str = concat!(
    "Mnelys/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/sdf123098/Mnelys)"
);

const REQUEST_TIMEOUT: Duration = Duration::from_secs(300);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);

/// A [`HttpClient`] over one shared `reqwest::Client`.
#[derive(Clone)]
pub struct ReqwestHttp {
    client: reqwest::Client,
}

impl ReqwestHttp {
    pub fn new() -> CoreResult<Self> {
        let client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .timeout(REQUEST_TIMEOUT)
            .connect_timeout(CONNECT_TIMEOUT)
            .build()
            .map_err(|error| {
                CoreError::internal("the HTTP client could not be created")
                    .with_details(error.to_string())
            })?;
        Ok(Self { client })
    }

    /// Wraps a client the caller configured, for tests and proxies.
    #[must_use]
    pub fn from_client(client: reqwest::Client) -> Self {
        Self { client }
    }

    #[must_use]
    pub fn inner(&self) -> &reqwest::Client {
        &self.client
    }
}

#[async_trait]
impl HttpClient for ReqwestHttp {
    async fn execute(&self, request: HttpRequest) -> CoreResult<HttpResponse> {
        let url = request.url.clone();
        let mut builder = match request.method {
            HttpMethod::Get => self.client.get(&url),
            HttpMethod::Post => self.client.post(&url),
        };

        if !request.headers.is_empty() {
            let mut headers = HeaderMap::new();
            for (name, value) in &request.headers {
                let name = HeaderName::from_bytes(name.as_bytes()).map_err(|error| {
                    CoreError::invalid_request(format!("invalid request header `{name}`"))
                        .with_details(error.to_string())
                })?;
                let value = HeaderValue::from_str(value).map_err(|error| {
                    CoreError::invalid_request(format!("invalid value for header `{name}`"))
                        .with_details(error.to_string())
                })?;
                headers.insert(name, value);
            }
            builder = builder.headers(headers);
        }

        builder = match request.body {
            Some(HttpBody::Json(value)) => builder.json(&value),
            Some(HttpBody::Form(fields)) => builder.form(&fields),
            None => builder,
        };

        let response = builder.send().await.map_err(|error| {
            let message = if error.is_timeout() {
                format!("{url} timed out")
            } else if error.is_connect() {
                format!("could not connect to {url}")
            } else {
                format!("the request to {url} failed")
            };
            CoreError::network(message).with_details(error.to_string())
        })?;

        let status = response.status().as_u16();
        let body = response.bytes().await.map_err(|error| {
            CoreError::network(format!("the response body from {url} was interrupted"))
                .with_details(error.to_string())
        })?;

        Ok(HttpResponse {
            status,
            body: body.to_vec(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_user_agent_names_the_product_and_version() {
        assert!(USER_AGENT.starts_with("Mnelys/"));
        assert!(USER_AGENT.contains(env!("CARGO_PKG_VERSION")));
    }

    #[test]
    fn a_client_can_be_constructed() {
        assert!(ReqwestHttp::new().is_ok());
    }
}
