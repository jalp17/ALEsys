use alesys_core::agent::protocol::{AgentCommand, AgentResponse, FileEntry};
use alesys_core::executor::{self, ExecutorConfig};
use alesys_core::fs_ops;
use clap::Parser;
use futures::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[derive(Parser)]
#[command(name = "alesys-agent")]
#[command(about = "ALEsys Remote Agent - Local execution bridge")]
struct Args {
    #[arg(short, long)]
    server: String,

    #[arg(short, long)]
    token: String,

    #[arg(short, long)]
    name: Option<String>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let name = args.name.unwrap_or_else(|| {
        hostname::get()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    });

    tracing::info!("Connecting to server: {}", args.server);

    loop {
        match connect_async(&args.server).await {
            Ok((ws_stream, _)) => {
                tracing::info!("Connected to server");

                let (mut write, mut read) = ws_stream.split();
                let (tx, mut rx) = mpsc::channel::<Vec<u8>>(32);

                let register = serde_json::json!({
                    "type": "register",
                    "payload": {
                        "name": name,
                        "token": args.token,
                    }
                });
                let _ = write.send(Message::Text(register.to_string().into())).await;

                // Handle incoming commands
                let write_task = tokio::spawn(async move {
                    while let Some(data) = rx.recv().await {
                        if write.send(Message::Binary(data.into())).await.is_err() {
                            break;
                        }
                    }
                });

                // Read commands and send responses
                while let Some(msg) = read.next().await {
                    if let Ok(Message::Text(text)) = msg {
                        if let Ok(command) = serde_json::from_str::<AgentCommand>(&text) {
                            let response = handle_command(command).await;
                if let Ok(data) = serde_json::to_vec(&response) {
                                let _ = tx.send(data).await;
                            }
                        }
                    }
                }

                write_task.abort();
                tracing::warn!("Disconnected from server, reconnecting in 5s...");
            }
            Err(e) => {
                tracing::error!("Connection failed: {}, retrying in 5s...", e);
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}

async fn handle_command(cmd: AgentCommand) -> AgentResponse {
    match cmd {
        AgentCommand::Execute { id, command, args, workdir, timeout_ms } => {
            let config = ExecutorConfig { timeout_ms, ..Default::default() };
            match executor::execute(
                &command,
                &args.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
                workdir.as_deref(),
                &config,
            ).await {
                Ok(result) => AgentResponse::ExecuteResult {
                    id,
                    exit_code: result.exit_code,
                    stdout: result.stdout,
                    stderr: result.stderr,
                    execution_time_ms: result.execution_time_ms,
                },
                Err(e) => AgentResponse::Error { id, message: e },
            }
        }
        AgentCommand::ReadFile { id, path } => {
            match fs_ops::read_file(&path).await {
                Ok(content) => AgentResponse::FileContent { id, content },
                Err(e) => AgentResponse::Error { id, message: e },
            }
        }
        AgentCommand::WriteFile { id, path, content } => {
            match fs_ops::write_file(&path, &content).await {
                Ok(()) => AgentResponse::Pong,
                Err(e) => AgentResponse::Error { id, message: e },
            }
        }
        AgentCommand::ListDirectory { id, path } => {
            match fs_ops::list_directory(&path).await {
                Ok(entries) => AgentResponse::DirectoryList {
                    id,
                    entries: entries.into_iter().map(|e| FileEntry {
                        name: e.name,
                        path: e.path,
                        is_dir: e.is_dir,
                        size: e.size,
                    }).collect(),
                },
                Err(e) => AgentResponse::Error { id, message: e },
            }
        }
        AgentCommand::Ping => AgentResponse::Pong,
    }
}
