use crate::db::Db;
use crate::mcp::{JsonRpcRequest, JsonRpcResponse};
use crate::prompts::{handle_get_prompt, handle_list_prompts};
use crate::resources::{handle_list_resources, handle_read_resource};
use crate::tools::{handle_call_tool, handle_list_tools};

use anyhow::Result;
use serde_json::json;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};

pub struct Server {
    db: Db,
}

impl Server {
    pub async fn new() -> Result<Self> {
        let db_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite://dev_insights.db".to_string());

        // Ensure the directory exists if using sqlite
        if db_url.starts_with("sqlite://") {
            if let Some(path_str) = db_url.strip_prefix("sqlite://") {
                if let Some(parent) = std::path::Path::new(path_str).parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
            }
        }

        let db = Db::new(&db_url).await?;
        Ok(Self { db })
    }

    pub async fn run(&self) -> Result<()> {
        let stdin = io::stdin();
        let reader = BufReader::new(stdin);
        let mut lines = reader.lines();

        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("/tmp/local-dev-insights-debug.log")
            .unwrap();

        while let Some(line) = lines.next_line().await? {
            use std::io::Write;
            writeln!(file, "Received: {}", line).unwrap();

            if line.trim().is_empty() {
                continue;
            }

            let request: JsonRpcRequest = match serde_json::from_str(&line) {
                Ok(req) => req,
                Err(e) => {
                    writeln!(file, "Failed to parse: {}", e).unwrap();
                    eprintln!("Failed to parse request: {}", e);
                    continue; // Client might show 'invalid request' if we don't reply?
                    // Actually, if we don't reply, it's a timeout.
                    // If we want to return invalid request:
                    // We can't because we don't have the ID.
                }
            };

            let response = self.handle_request(request).await;

            // JSON-RPC 2.0: Server MUST NOT reply to a Notification.
            if response.id.is_none() && response.result.is_none() && response.error.is_none() {
                continue;
            }

            let response_json = serde_json::to_string(&response)?;

            writeln!(file, "Sending: {}", response_json).unwrap();

            let mut stdout = io::stdout();
            stdout.write_all(response_json.as_bytes()).await?;
            stdout.write_all(b"\n").await?;
            stdout.flush().await?;
        }

        Ok(())
    }

    async fn handle_request(&self, req: JsonRpcRequest) -> JsonRpcResponse {
        match req.method.as_str() {
            "initialize" => {
                JsonRpcResponse::success(
                    req.id,
                    json!({
                        "protocolVersion": "2024-11-05", // Updated to match modern MCP spec expectation
                        "capabilities": {
                            "resources": {},
                            "tools": {},
                            "prompts": {}
                        },
                        "serverInfo": {
                            "name": "local-dev-insights",
                            "version": "0.1.0"
                        }
                    }),
                )
            }
            "ping" => JsonRpcResponse::success(req.id, json!({})),
            "resources/list" => {
                let result = handle_list_resources(&self.db).await;
                JsonRpcResponse::success(req.id, result)
            }
            "resources/read" => match handle_read_resource(&self.db, req.params).await {
                Ok(result) => JsonRpcResponse::success(req.id, result),
                Err(e) => JsonRpcResponse::error(req.id, -32602, e, None),
            },
            "tools/list" => {
                let result = handle_list_tools().await;
                JsonRpcResponse::success(req.id, result)
            }
            "tools/call" => {
                let params = req.params.unwrap_or(json!({}));
                let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let arguments = params.get("arguments").cloned();

                match handle_call_tool(&self.db, name, arguments).await {
                    Ok(result) => JsonRpcResponse::success(req.id, result),
                    Err(e) => JsonRpcResponse::error(req.id, -32602, e, None),
                }
            }
            "prompts/list" => {
                let result = handle_list_prompts().await;
                JsonRpcResponse::success(req.id, result)
            }
            "prompts/get" => {
                let params = req.params.unwrap_or(json!({}));
                let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let arguments = params.get("arguments").cloned();

                match handle_get_prompt(&self.db, name, arguments).await {
                    Ok(result) => JsonRpcResponse::success(req.id, result),
                    Err(e) => JsonRpcResponse::error(req.id, -32602, e, None),
                }
            }
            "notifications/initialized" => {
                // MCP lifecycle notification, just ignore
                // Do not return a response for notifications
                return JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: None,
                    id: None,
                };
            }
            _ => {
                if req.id.is_none() {
                    // Ignore unknown notifications
                    return JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: None,
                        error: None,
                        id: None,
                    };
                }
                JsonRpcResponse::error(
                    req.id,
                    -32601,
                    format!("Method not found: {}", req.method),
                    None,
                )
            }
        }
    }
}
