use crate::db::Db;
use serde_json::{Value, json};
use std::fs;

pub async fn handle_list_resources(_db: &Db) -> Value {
    // Return list of available resources
    json!({
        "resources": [
            {
                "uri": "db://memos",
                "name": "Development Memos",
                "description": "List of all development memos stored in the database",
                "mimeType": "application/json"
            },
            {
                "uri": "env://vars",
                "name": "Environment Variables",
                "description": "Environment variables from .env file",
                "mimeType": "application/json"
            }
        ]
    })
}

pub async fn handle_read_resource(db: &Db, params: Option<Value>) -> Result<Value, String> {
    let params = params.ok_or("Missing params")?;
    let uri = params
        .get("uri")
        .and_then(|v| v.as_str())
        .ok_or("Missing uri parameter")?;

    match uri {
        "db://memos" => {
            let memos = db.list_memos().await.map_err(|e| e.to_string())?;
            Ok(json!({
                "contents": [{
                    "uri": uri,
                    "mimeType": "application/json",
                    "text": serde_json::to_string(&memos).unwrap()
                }]
            }))
        }
        "env://vars" => {
            // Read .env file manually or dump env vars. Spec says "from .env".
            // Since dotenv is loaded, we can just look for .env file or iterate env vars.
            // Iterating env vars might leak sensitive system info if not careful, but usually local dev insights implies access to local env.
            // Spec says: "Read environment variables list from project root .env".
            // So I should read the file directly.

            let env_content = fs::read_to_string(".env").unwrap_or_else(|_| "".to_string());
            Ok(json!({
                "contents": [{
                    "uri": uri,
                    "mimeType": "text/plain", // or application/json if we parse it? Spec says "list of env vars".
                    // "description: Read environment variables list from project root .env"
                    // Usually resources return text.
                    "text": env_content
                }]
            }))
        }
        _ => Err(format!("Resource not found: {}", uri)),
    }
}
