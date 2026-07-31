use std::io::{self, BufRead, Write};

use easytube_core::provider::{BinaryProvider, ProviderType};
use easytube_core::state::{DownloadManager, DownloadSettings};
use easytube_core::types::JobStatus;
use serde_json::{json, Value};
use std::sync::Arc;

pub fn serve() {
    eprintln!("EasyTube MCP server v0.1.0 (stdio transport)");

    let ytdlp = BinaryProvider::find_ytdlp(&ProviderType::System)
        .or_else(|| BinaryProvider::find_ytdlp(&ProviderType::Downloaded))
        .unwrap_or_else(|| std::path::PathBuf::from("yt-dlp"));

    let ffmpeg = BinaryProvider::find_ffmpeg(&ProviderType::System)
        .or_else(|| BinaryProvider::find_ffmpeg(&ProviderType::Downloaded));

    let manager = Arc::new(DownloadManager::new(
        DownloadSettings::default(),
        ytdlp,
        ffmpeg,
    ));

    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");

    let stdin = std::io::stdin();
    let mut reader = stdin.lock();
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {}
            Err(e) => {
                eprintln!("stdin error: {}", e);
                break;
            }
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let request: Value = match serde_json::from_str(trimmed) {
            Ok(r) => r,
            Err(e) => {
                let err = json!({
                    "jsonrpc": "2.0",
                    "id": null,
                    "error": {"code": -32700, "message": "Parse error", "data": e.to_string()}
                });
                println!("{}", err);
                continue;
            }
        };

        let id = request.get("id").cloned().unwrap_or(Value::Null);
        let method = request.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let params = request.get("params").cloned().unwrap_or(Value::Null);

        let response = match method {
            "initialize" => handle_initialize(id),
            "tools/list" => handle_list_tools(id),
            "tools/call" => {
                let mgr = manager.clone();
                rt.block_on(handle_call_tool(id, &params, mgr))
            }
            "notifications/initialized" => continue,
            _ => json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {"code": -32601, "message": format!("Method not found: {}", method)}
            }),
        };

        println!("{}", serde_json::to_string(&response).unwrap());
    }
}

fn handle_initialize(id: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "serverInfo": {
                "name": "easytube",
                "version": "0.1.0"
            }
        }
    })
}

fn handle_list_tools(id: Value) -> Value {
    let tools = vec![
        tool_def(
            "easytube_probe",
            "Probe a video URL and return metadata (title, formats, thumbnail, duration).",
            json!({
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "The video URL to probe (YouTube, Bilibili, etc.)"
                    }
                },
                "required": ["url"]
            }),
        ),
        tool_def(
            "easytube_download",
            "Start downloading a video. Returns a job_id for tracking progress.",
            json!({
                "type": "object",
                "properties": {
                    "url": {"type": "string", "description": "The video URL"},
                    "format_id": {"type": "string", "description": "Format ID (leave empty for best quality)"},
                    "output_dir": {"type": "string", "description": "Custom download directory"}
                },
                "required": ["url"]
            }),
        ),
        tool_def(
            "easytube_progress",
            "Get the progress of a download job by job_id.",
            json!({
                "type": "object",
                "properties": {
                    "job_id": {"type": "string", "description": "The job ID from easytube_download"}
                },
                "required": ["job_id"]
            }),
        ),
        tool_def(
            "easytube_stop",
            "Stop/cancel a running download.",
            json!({
                "type": "object",
                "properties": {
                    "job_id": {"type": "string"}
                },
                "required": ["job_id"]
            }),
        ),
        tool_def(
            "easytube_settings",
            "Get or update settings. Provide key and optional value to set.",
            json!({
                "type": "object",
                "properties": {
                    "key": {"type": "string", "description": "Setting key"},
                    "value": {"type": "string", "description": "New value (omit to read current value)"}
                },
                "required": ["key"]
            }),
        ),
    ];

    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "tools": tools
        }
    })
}

fn tool_def(name: &str, description: &str, input_schema: Value) -> Value {
    json!({
        "name": name,
        "description": description,
        "inputSchema": input_schema
    })
}

async fn handle_call_tool(
    id: Value,
    params: &Value,
    manager: Arc<DownloadManager>,
) -> Value {
    let name = params
        .get("name")
        .and_then(|n| n.as_str())
        .unwrap_or("");

    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or(Value::Null);

    let result = match name {
        "easytube_probe" => {
            let url = arguments
                .get("url")
                .and_then(|u| u.as_str())
                .unwrap_or("");
            match manager.probe(url).await {
                Ok(info) => serde_json::to_string_pretty(&info).unwrap(),
                Err(e) => format!("Error: {}", e),
            }
        }
        "easytube_download" => {
            let url = arguments
                .get("url")
                .and_then(|u| u.as_str())
                .unwrap_or("")
                .to_string();
            let format_id = arguments
                .get("format_id")
                .and_then(|f| f.as_str())
                .map(|s| s.to_string());
            let output_dir = arguments
                .get("output_dir")
                .and_then(|d| d.as_str())
                .map(|s| std::path::PathBuf::from(s));

            match manager.create_download(url, format_id, output_dir).await {
                Ok(job_id) => {
                    // Start download in background
                    let mgr = manager.clone();
                    let jid = job_id.clone();
                    tokio::spawn(async move {
                        let _ = start_download_task(&mgr, &jid).await;
                    });
                    format!("Download started. job_id: {}", job_id)
                }
                Err(e) => format!("Error: {}", e),
            }
        }
        "easytube_progress" => {
            let job_id = arguments
                .get("job_id")
                .and_then(|j| j.as_str())
                .unwrap_or("");
            match manager.get_job(job_id).await {
                Some(job) => serde_json::to_string_pretty(&json!({
                    "job_id": job.id,
                    "status": format!("{:?}", job.status),
                    "title": job.title,
                    "progress": job.progress,
                }))
                .unwrap(),
                None => format!("Job not found: {}", job_id),
            }
        }
        "easytube_stop" => {
            let job_id = arguments
                .get("job_id")
                .and_then(|j| j.as_str())
                .unwrap_or("");
            manager.update_status(job_id, JobStatus::Cancelled).await;
            format!("Job {} cancelled", job_id)
        }
        "easytube_settings" => {
            let key = arguments
                .get("key")
                .and_then(|k| k.as_str())
                .unwrap_or("");
            let _value = arguments.get("value").and_then(|v| v.as_str());
            format!("Settings for '{}': not yet persisted", key)
        }
        _ => format!("Unknown tool: {}", name),
    };

    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "content": [{
                "type": "text",
                "text": result
            }]
        }
    })
}

async fn start_download_task(manager: &Arc<DownloadManager>, job_id: &str) -> Result<(), String> {
    manager.update_status(job_id, JobStatus::Probe).await;

    let job = manager.get_job(job_id).await.ok_or("job not found")?;
    let info = manager.probe(&job.url).await?;

    manager.update_status(job_id, JobStatus::Downloading).await;
    let title = info.title.clone();

    let out_dir = job
        .output_dir
        .unwrap_or_else(|| manager.output_dir().clone())
        .to_string_lossy()
        .to_string();

    let fmt = job.format_id.as_deref().unwrap_or("best");

    let output = tokio::process::Command::new(manager.ytdlp_path())
        .args([
            "-o",
            &format!("{}/%(title)s.%(ext)s", out_dir),
            "-f",
            fmt,
            "--no-playlist",
            "--ignore-config",
            &job.url,
        ])
        .output()
        .await;

    match output {
        Ok(o) if o.status.success() => {
            manager.update_status(job_id, JobStatus::Done).await;
            eprintln!("MCP download done: {}", title);
            Ok(())
        }
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            manager
                .update_status(job_id, JobStatus::Failed(stderr.to_string()))
                .await;
            Err(stderr.to_string())
        }
        Err(e) => {
            manager
                .update_status(job_id, JobStatus::Failed(e.to_string()))
                .await;
            Err(e.to_string())
        }
    }
}
