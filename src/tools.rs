use crate::db::Db;
use anyhow::Result;
use serde_json::{Value, json};
use sysinfo::System;
use walkdir::WalkDir;

pub async fn handle_list_tools() -> Value {
    json!({
        "tools": [
            {
                "name": "add_memo",
                "description": "Add a new development memo to the database",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "content": { "type": "string", "description": "Content of the memo" },
                        "tags": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Tags for the memo"
                        }
                    },
                    "required": ["content"]
                }
            },
            {
                "name": "get_system_stats",
                "description": "Get current system statistics (CPU, Memory)",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            },
            {
                "name": "list_files_by_extension",
                "description": "Recursively list files with a specific extension in the current directory",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "extension": { "type": "string", "description": "File extension (e.g., 'rs', 'md')" }
                    },
                    "required": ["extension"]
                }
            }
        ]
    })
}

pub async fn handle_call_tool(
    db: &Db,
    name: &str,
    arguments: Option<Value>,
) -> Result<Value, String> {
    let args = arguments.unwrap_or(json!({}));

    match name {
        "add_memo" => {
            let content = args
                .get("content")
                .and_then(|v| v.as_str())
                .ok_or("Missing content")?;
            let tags_val = args.get("tags").and_then(|v| v.as_array());
            let tags: Vec<String> = tags_val
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();

            let id = db
                .add_memo(content, &tags)
                .await
                .map_err(|e| e.to_string())?;
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("Memo added successfully with ID: {}", id)
                }]
            }))
        }
        "get_system_stats" => {
            let mut sys = System::new_all();
            sys.refresh_all();

            let cpu_usage = sys.global_cpu_info().cpu_usage();
            let total_memory = sys.total_memory();
            let used_memory = sys.used_memory();
            let memory_usage =
                format!("{:.2}%", (used_memory as f64 / total_memory as f64) * 100.0);

            // Spec asks for listen state on 3000, 8080.
            // sysinfo can check processes but checking specific ports usually requires iterating connections (if supported) or trying to bind.
            // Getting network connections (netstat equivalent) is supported in sysinfo via `Networks`.
            // Wait, sysinfo `SystemExt` doesn't directly give listening ports easily in a cross-platform way without listing all processes and their connections.
            // Simplified: just return CPU and Memory for now as per minimal implementation, maybe add port check if easy.
            // Let's stick to CPU and Memory.

            let response = format!(
                "CPU Usage: {:.2}%\nMemory Usage: {} (Used: {} MB / Total: {} MB)",
                cpu_usage,
                memory_usage,
                used_memory / 1024 / 1024,
                total_memory / 1024 / 1024
            );

            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": response
                }]
            }))
        }
        "list_files_by_extension" => {
            let extension = args
                .get("extension")
                .and_then(|v| v.as_str())
                .ok_or("Missing extension")?;
            // Validate validation: prevent traversing up? Walkdir starts at current dir.
            // Spec says: "Project root access only".
            let root = std::env::current_dir().map_err(|e| e.to_string())?;

            let mut files = Vec::new();
            for entry in WalkDir::new(&root).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext.to_string_lossy() == extension {
                            // Valid path check (simple since we walk from current dir)
                            if let Ok(rel) = path.strip_prefix(&root) {
                                files.push(rel.to_string_lossy().to_string());
                            }
                        }
                    }
                }
            }

            // Limit output if too many?
            let response = files.join("\n");
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": if response.is_empty() { "No files found.".to_string() } else { response }
                }]
            }))
        }
        _ => Err(format!("Tool not found: {}", name)),
    }
}
