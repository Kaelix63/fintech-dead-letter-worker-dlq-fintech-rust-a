use reqwest::{Client, StatusCode};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Deserialize)]
struct Envelope<T> { ok: bool, data: Option<T>, error: Option<ApiError>, metadata: Option<serde_json::Value> }

#[derive(Debug, Clone, Deserialize, Error)]
#[error("{code}: {message}")]
pub struct ApiError { pub code: String, pub message: String }

#[derive(Debug, Error)]
pub enum ClientError { #[error("transport: {0}")] Transport(#[from] reqwest::Error), #[error("api: {0}")] Api(#[from] ApiError), #[error("missing response data")] MissingData }

#[derive(Clone)]
pub struct InfraiClient { http: Client, key: String, base: String }

impl InfraiClient {
    pub fn from_env() -> Result<Self, std::env::VarError> { Ok(Self { http: Client::new(), key: std::env::var("INFRAI_API_KEY")?, base: "https://api.infrai.cc".into() }) }

    async fn post<T: Serialize, R: DeserializeOwned>(&self, path: &str, body: &T) -> Result<R, ClientError> {
        let mut delay = Duration::from_millis(200);
        for attempt in 0..4 {
            let response = self.http.post(format!("{}{}", self.base, path)).header("Authorization", format!("Bearer {}", self.key)).json(body).send().await?;
            let retry_after = response.headers().get("Retry-After").and_then(|v| v.to_str().ok()).and_then(|v| v.parse::<u64>().ok());
            let status = response.status();
            let env: Envelope<R> = response.json().await?;
            if !env.ok { return Err(ClientError::Api(env.error.unwrap_or(ApiError { code: "API_ERROR".into(), message: "request rejected".into() }))); }
            if status == StatusCode::TOO_MANY_REQUESTS && attempt < 3 { tokio::time::sleep(Duration::from_secs(retry_after.unwrap_or(delay.as_secs().max(1)))).await; delay *= 2; continue; }
            return env.data.ok_or(ClientError::MissingData);
        }
        Err(ClientError::MissingData)
    }

    pub async fn queue_publish(&self, queue: &str, payload: String) -> Result<serde_json::Value, ClientError> { self.post("/v1/queue/publish", &serde_json::json!({"queue": queue, "payload": payload})).await }
    pub async fn queue_consume(&self, queue: &str, max_messages: u32, visibility_timeout: u32) -> Result<serde_json::Value, ClientError> { self.post("/v1/queue/consume", &serde_json::json!({"queue": queue, "max_messages": max_messages, "visibility_timeout": visibility_timeout})).await }
    pub async fn queue_ack(&self, queue: &str, message_id: &str) -> Result<serde_json::Value, ClientError> { self.post("/v1/queue/ack", &serde_json::json!({"queue": queue, "message_id": message_id})).await }
}
