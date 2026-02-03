use crate::db::Db;
use serde_json::{Value, json};
use sysinfo::System;

pub async fn handle_list_prompts() -> Value {
    json!({
        "prompts": [
            {
                "name": "analyze-health",
                "description": "Analyze system health based on current stats",
                "arguments": []
            }
        ]
    })
}

pub async fn handle_get_prompt(
    _db: &Db,
    name: &str,
    _arguments: Option<Value>,
) -> Result<Value, String> {
    match name {
        "analyze-health" => {
            let mut sys = System::new_all();
            sys.refresh_all();

            let cpu_usage = sys.global_cpu_info().cpu_usage();
            let total_memory = sys.total_memory();
            let used_memory = sys.used_memory();
            let memory_usage_percent = (used_memory as f64 / total_memory as f64) * 100.0;

            let stats = format!(
                "CPU Usage: {:.2}%\nMemory Usage: {:.2}% (Used: {} MB / Total: {} MB)",
                cpu_usage,
                memory_usage_percent,
                used_memory / 1024 / 1024,
                total_memory / 1024 / 1024
            );

            Ok(json!({
                "description": "Analyze system health",
                "messages": [
                    {
                        "role": "user",
                        "content": {
                            "type": "text",
                            "text": format!("Here are the current system statistics:\n\n{}\n\nPlease analyze if the development environment is healthy or if there are any resource constraints I should be aware of.", stats)
                        }
                    }
                ]
            }))
        }
        _ => Err(format!("Prompt not found: {}", name)),
    }
}
