//! Varlink client for communicating with contextd over Unix domain sockets.

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

/// Maximum per-call reply buffer size in bytes.
pub const MAX_MSG_BYTES: usize = 1024 * 1024;

/// Low-level Varlink request payload.
#[derive(Debug, Serialize)]
struct VarlinkRequest<'a> {
    method: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameters: Option<Value>,
}

/// Low-level Varlink response envelope.
#[derive(Debug, Deserialize)]
struct VarlinkResponse {
    parameters: Option<Value>,
    error: Option<String>,
}

/// Client for issuing Varlink calls to contextd.
pub struct ContextdClient {
    socket_path: String,
}

impl ContextdClient {
    /// Creates a client targeting a designated Unix domain socket.
    pub fn new<P: AsRef<Path>>(socket_path: P) -> Self {
        Self {
            socket_path: socket_path.as_ref().to_string_lossy().to_string(),
        }
    }

    /// Invokes a Varlink method and awaits a single reply.
    pub async fn call(&self, method: &str, parameters: Option<Value>) -> Result<Value> {
        let mut stream = UnixStream::connect(&self.socket_path)
            .await
            .with_context(|| format!("Failed to connect to contextd socket at {}", self.socket_path))?;

        let request = VarlinkRequest { method, parameters };
        let mut req_bytes = serde_json::to_vec(&request)?;
        req_bytes.push(0x00);

        stream.write_all(&req_bytes).await?;

        let mut buffer = Vec::with_capacity(4096);
        let mut chunk = [0u8; 1024];

        loop {
            let n = stream.read(&mut chunk).await?;
            if n == 0 {
                bail!("Connection closed by contextd before complete Varlink reply");
            }
            if buffer.len() + n > MAX_MSG_BYTES {
                bail!(
                    "Varlink reply exceeded {} byte cap; aborting",
                    MAX_MSG_BYTES
                );
            }
            buffer.extend_from_slice(&chunk[..n]);

            if let Some(pos) = buffer.iter().position(|&b| b == 0x00) {
                let reply_bytes = &buffer[..pos];
                let response: VarlinkResponse = serde_json::from_slice(reply_bytes)
                    .context("Failed parsing Varlink reply payload")?;

                if let Some(err) = response.error {
                    // Include the parameters payload when present so the
                    // operator can see *why* the daemon rejected the
                    // call (InvalidParameter reason, NotFound id, etc.)
                    // rather than just the error name.
                    let details = response
                        .parameters
                        .as_ref()
                        .map(|p| p.to_string())
                        .unwrap_or_default();
                    if details.is_empty() {
                        bail!("Varlink error returned: {}", err);
                    } else {
                        bail!("Varlink error returned: {} ({})", err, details);
                    }
                }

                return Ok(response.parameters.unwrap_or(Value::Null));
            }
        }
    }
}