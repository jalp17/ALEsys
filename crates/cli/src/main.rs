//! ALEsys CLI - Interfaz de linea de comandos

use anyhow::Result;
use clap::{Parser, Subcommand};
use sqlx::Row;

mod output;
mod ingestion;

#[derive(Parser)]
#[command(name = "alesys")]
#[command(about = "ALEsys GraphRAG-PG CLI")]
#[command(long_about = "Command-line interface for ALEsys GraphRAG-PG.\nManage database, sessions, LLM, Docker services, and more.")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Verbose output
    #[arg(short, long, global = true, default_value_t = false)]
    verbose: bool,

    /// Output as JSON (machine-readable)
    #[arg(long, global = true, default_value_t = false)]
    json: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Database management
    Db {
        #[command(subcommand)]
        action: DbCommands,
    },

    /// Session management
    Session {
        #[command(subcommand)]
        action: SessionCommands,
    },

    /// LLM model management
    Llm {
        #[command(subcommand)]
        action: LlmCommands,
    },

    /// Docker service management
    Docker {
        #[command(subcommand)]
        action: DockerCommands,
    },

    /// PDF ingestion management
    Ingest {
        #[command(subcommand)]
        action: ingestion::IngestionCommand,
    },

    /// Show system status
    Status,

    /// Show current configuration
    Config,

    /// Graph operations
    Graph {
        #[command(subcommand)]
        action: GraphCommands,
    },

    /// Search indexed documents
    Search {
        /// Search query
        query: String,

        /// Max results
        #[arg(short, long, default_value_t = 10)]
        limit: usize,
    },

    /// List indexed documents
    List,

    /// Chat with RAG context
    Ask {
        /// Your question
        question: String,

        /// Session ID (creates new if not provided)
        #[arg(short, long)]
        session: Option<String>,
    },
}

#[derive(Subcommand)]
enum DbCommands {
    /// Initialize database (create tables)
    Init,

    /// Drop all tables (with confirmation)
    Drop {
        /// Skip confirmation
        #[arg(long, default_value_t = false)]
        force: bool,
    },

    /// Run pending migrations
    Migrate,

    /// Show migration status
    MigrateStatus,
}

#[derive(Subcommand)]
enum SessionCommands {
    /// Create a new session
    New {
        /// Session name
        #[arg(short, long)]
        name: Option<String>,

        /// User ID
        #[arg(long, default_value_t = 0)]
        user_id: i32,
    },

    /// List active sessions
    List {
        /// User ID
        #[arg(long, default_value_t = 0)]
        user_id: i32,
    },

    /// Close a session
    Close {
        /// Session ID
        session_id: String,
    },

    /// Show session history
    History {
        /// Session ID
        session_id: String,

        /// Max messages to show
        #[arg(short, long, default_value_t = 20)]
        limit: usize,
    },
}

#[derive(Subcommand)]
enum LlmCommands {
    /// Show LLM status
    Status,

    /// Load LLM model into memory
    Load {
        /// Force reload even if already loaded
        #[arg(long, default_value_t = false)]
        force: bool,
    },

    /// Unload LLM model (free RAM)
    Unload,
}

#[derive(Subcommand)]
enum DockerCommands {
    /// Start all services
    Up {
        /// Build images before starting
        #[arg(long, default_value_t = false)]
        build: bool,

        /// Services to start (default: all)
        #[arg(trailing_var_arg = true)]
        services: Vec<String>,
    },

    /// Stop all services
    Down {
        /// Remove volumes too
        #[arg(long, default_value_t = false)]
        volumes: bool,
    },

    /// Restart services
    Restart {
        /// Specific service to restart
        service: Option<String>,
    },

    /// Show service logs
    Logs {
        /// Service name
        service: Option<String>,

        /// Follow log output
        #[arg(short, long, default_value_t = false)]
        follow: bool,

        /// Lines to show
        #[arg(short, long, default_value_t = 50)]
        lines: usize,
    },

    /// Show service status
    Ps,
}

#[derive(Subcommand)]
enum GraphCommands {
    /// Show graph statistics
    Stats,

    /// Export graph to JSON
    Export {
        /// Output file path
        #[arg(short, long, default_value = "graph-export.json")]
        output: String,
    },

    /// Find shortest path between two documents
    Path {
        /// Source document ID
        from: i32,

        /// Target document ID
        to: i32,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.verbose {
        std::env::set_var("RUST_LOG", "debug");
    }

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "alesys_cli=info".into()),
        )
        .init();

    dotenvy::dotenv().ok();

    match cli.command {
        Commands::Db { action } => handle_db(action, cli.json).await,
        Commands::Session { action } => handle_session(action, cli.json).await,
        Commands::Llm { action } => handle_llm(action, cli.json).await,
        Commands::Docker { action } => handle_docker(action, cli.json).await,
        Commands::Ingest { action } => {
            ingestion::handle_ingestion(ingestion::IngestionArgs { command: action }).await
        }
        Commands::Status => handle_status(cli.json).await,
        Commands::Config => handle_config(cli.json),
        Commands::Graph { action } => handle_graph(action, cli.json).await,
        Commands::Search { query, limit } => handle_search(&query, limit, cli.json).await,
        Commands::List => handle_list(cli.json).await,
        Commands::Ask {
            question,
            session,
        } => handle_ask(&question, session.as_deref(), cli.json).await,
    }
}

// ── Database commands ──────────────────────────────────────────────

async fn handle_db(action: DbCommands, _json: bool) -> Result<()> {
    match action {
        DbCommands::Init => {
            output::header("Initializing database");
            let pool = alesys_core::db::create_db_pool().await?;
            output::info("Connected to PostgreSQL");

            let sql = include_str!("../../../docker/init-db.sql");
            alesys_core::db::execute_sql(&pool, sql).await?;
            output::success("Core tables created");

            let sql6 = include_str!("../../../docker/init-db-6.sql");
            if let Err(e) = alesys_core::db::execute_sql(&pool, sql6).await {
                output::warn(&format!("Search indexes partial failure: {}", e));
            } else {
                output::success("Search indexes created");
            }

            let migration = include_str!("../../../docker/migrations/graph_permissions.sql");
            if let Err(e) = alesys_core::db::execute_sql(&pool, migration).await {
                output::warn(&format!("Migration partial failure: {}", e));
            } else {
                output::success("Migrations applied");
            }

            output::success("Database initialized successfully");
        }

        DbCommands::Drop { force } => {
            if !force {
                print!("Are you sure you want to DROP all tables? [y/N]: ");
                use std::io::Write;
                std::io::stdout().flush()?;
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                if input.trim().to_lowercase() != "y" {
                    output::info("Cancelled");
                    return Ok(());
                }
            }

            output::header("Dropping database tables");
            let pool = alesys_core::db::create_db_pool().await?;

            let tables = [
                "session_context",
                "session_messages",
                "user_sessions",
                "graph_permissions",
                "enlaces",
                "relaciones",
                "entidades",
                "fragmentos",
                "documentos",
            ];

            for table in &tables {
                match sqlx::query(&format!("DROP TABLE IF EXISTS {} CASCADE", table))
                    .execute(&pool)
                    .await
                {
                    Ok(_) => output::success(&format!("Dropped table: {}", table)),
                    Err(e) => output::warn(&format!("Failed to drop {}: {}", table, e)),
                }
            }

            output::success("All tables dropped");
        }

        DbCommands::Migrate => {
            output::header("Running migrations");
            let pool = alesys_core::db::create_db_pool().await?;

            sqlx::query(
                r#"CREATE TABLE IF NOT EXISTS _migrations (
                    id SERIAL PRIMARY KEY,
                    name VARCHAR(255) UNIQUE NOT NULL,
                    applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                )"#,
            )
            .execute(&pool)
            .await?;

            let migrations: Vec<(&str, &str)> = vec![
                ("graph_permissions", include_str!("../../../docker/migrations/graph_permissions.sql")),
            ];

            for (name, sql) in &migrations {
                let already_applied = sqlx::query_scalar::<_, i64>(
                    "SELECT COUNT(*) FROM _migrations WHERE name = $1",
                )
                .bind(name)
                .fetch_one(&pool)
                .await?
                    > 0;

                if already_applied {
                    output::info(&format!("Already applied: {}", name));
                    continue;
                }

                match alesys_core::db::execute_sql(&pool, sql).await {
                    Ok(_) => {
                        sqlx::query("INSERT INTO _migrations (name) VALUES ($1)")
                            .bind(name)
                            .execute(&pool)
                            .await?;
                        output::success(&format!("Applied migration: {}", name));
                    }
                    Err(e) => output::warn(&format!("Migration {} failed: {}", name, e)),
                }
            }

            output::success("Migrations complete");
        }

        DbCommands::MigrateStatus => {
            let pool = alesys_core::db::create_db_pool().await?;

            sqlx::query(
                r#"CREATE TABLE IF NOT EXISTS _migrations (
                    id SERIAL PRIMARY KEY,
                    name VARCHAR(255) UNIQUE NOT NULL,
                    applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                )"#,
            )
            .execute(&pool)
            .await?;

            let rows = sqlx::query("SELECT name, applied_at FROM _migrations ORDER BY id")
                .fetch_all(&pool)
                .await?;

            if rows.is_empty() {
                output::info("No migrations applied yet");
                return Ok(());
            }

            output::header("Applied Migrations");
            let mut table = output::new_table(&["Name", "Applied At"]);
            for row in &rows {
                table.add_row(vec![
                    row.get::<String, _>(0),
                    row.get::<chrono::NaiveDateTime, _>(1).to_string(),
                ]);
            }
            println!("{}", table);
        }
    }

    Ok(())
}

// ── Session commands ───────────────────────────────────────────────

async fn handle_session(action: SessionCommands, json: bool) -> Result<()> {
    let pool = alesys_core::db::create_db_pool().await?;
    let manager = alesys_core::SessionManager::new(pool);

    match action {
        SessionCommands::New { name, user_id } => {
            let id = manager.create_session(user_id, name.clone()).await?;
            if json {
                println!(
                    "{}",
                    serde_json::json!({ "session_id": id, "name": name })
                );
            } else {
                output::success(&format!("Session created: {}", id));
            }
        }

        SessionCommands::List { user_id } => {
            let sessions = manager.get_active_sessions(user_id).await?;

            if json {
                let data: Vec<_> = sessions
                    .iter()
                    .map(|s| {
                        serde_json::json!({
                            "id": s.id,
                            "name": s.name,
                            "created_at": s.created_at.to_rfc3339(),
                            "last_activity": s.last_activity.to_rfc3339(),
                        })
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&data)?);
            } else {
                if sessions.is_empty() {
                    output::info("No active sessions");
                    return Ok(());
                }
                output::header("Active Sessions");
                let mut table = output::new_table(&["ID", "Name", "Last Activity"]);
                for s in &sessions {
                    table.add_row(vec![
                        &s.id[..8],
                        &s.name,
                        &s.last_activity.format("%Y-%m-%d %H:%M").to_string(),
                    ]);
                }
                println!("{}", table);
            }
        }

        SessionCommands::Close { session_id } => {
            manager.close_session(&session_id).await?;
            if json {
                println!("{}", serde_json::json!({ "closed": session_id }));
            } else {
                output::success(&format!("Session closed: {}", session_id));
            }
        }

        SessionCommands::History { session_id, limit } => {
            let messages = manager.get_session_history(&session_id, limit).await?;

            if json {
                let data: Vec<_> = messages
                    .iter()
                    .map(|m| {
                        serde_json::json!({
                            "role": m.role,
                            "content": m.content,
                            "timestamp": m.timestamp.to_rfc3339(),
                        })
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&data)?);
            } else {
                if messages.is_empty() {
                    output::info("No messages in session");
                    return Ok(());
                }
                output::header(&format!("Session History ({})", &session_id[..8]));
                for m in &messages {
                    let role_color = match m.role.as_str() {
                        "user" => "\x1b[34m",
                        "assistant" => "\x1b[32m",
                        _ => "\x1b[0m",
                    };
                    println!(
                        "{}[{}]\x1b[0m {}",
                        role_color,
                        m.role,
                        m.content.chars().take(200).collect::<String>()
                    );
                }
            }
        }
    }

    Ok(())
}

// ── LLM commands ──────────────────────────────────────────────────

async fn handle_llm(action: LlmCommands, json: bool) -> Result<()> {
    match action {
        LlmCommands::Status => {
            let config = alesys_core::llm::LLMConfig::from_env();
            let backend = alesys_core::llm::LLMBackend::from_config_lazy(config.clone()).await;

            let (state_str, backend_name) = match backend {
                Ok(b) => {
                    use alesys_core::llm::LLMEngine;
                    (format!("{:?}", b.state()), b.backend_name().to_string())
                }
                Err(_) => ("unavailable".to_string(), "none".to_string()),
            };

            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "backend": backend_name,
                        "state": state_str,
                        "model": config.model_path,
                        "context_size": config.context_size,
                    })
                );
            } else {
                output::header("LLM Status");
                output::kv("Backend", &backend_name);
                output::kv("State", &state_str);
                output::kv("Model", &config.model_path);
                output::kv("Context Size", &config.context_size);
                output::kv("Max Tokens", &config.max_tokens);
            }
        }

        LlmCommands::Load { force } => {
            let config = alesys_core::llm::LLMConfig::from_env();
            let mut backend = alesys_core::llm::LLMBackend::from_config_lazy(config.clone()).await?;

            if backend.is_loaded() && !force {
                output::info("Model already loaded (use --force to reload)");
                return Ok(());
            }

            output::info("Loading LLM model into memory...");
            let cfg = alesys_core::llm::LLMConfig::from_env();
            backend.load(&cfg).await?;
            output::success("LLM model loaded");
        }

        LlmCommands::Unload => {
            let config = alesys_core::llm::LLMConfig::from_env();
            let mut backend = alesys_core::llm::LLMBackend::from_config_lazy(config).await?;

            if !backend.is_loaded() {
                output::info("Model already unloaded");
                return Ok(());
            }

            backend.unload().await?;
            output::success("LLM model unloaded (RAM freed)");
        }
    }

    Ok(())
}

// ── Docker commands ────────────────────────────────────────────────

fn docker_compose(args: &[&str]) -> Result<std::process::Output> {
    use std::process::Command;
    let output = Command::new("docker")
        .args(["compose", "-f", "docker/docker-compose.yml"])
        .args(args)
        .output()?;
    Ok(output)
}

async fn handle_docker(action: DockerCommands, _json: bool) -> Result<()> {
    match action {
        DockerCommands::Up { build, services } => {
            let mut args = vec!["up", "-d"];
            if build {
                args.push("--build");
            }
            let service_refs: Vec<&str> = services.iter().map(|s| s.as_str()).collect();
            args.extend_from_slice(&service_refs);

            let output = docker_compose(&args)?;
            if output.status.success() {
                output::success("Services started");
                if !service_refs.is_empty() {
                    output::info(&format!("Services: {}", service_refs.join(", ")));
                }
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                output::error(&format!("Failed to start services: {}", stderr));
            }
        }

        DockerCommands::Down { volumes } => {
            let mut args = vec!["down"];
            if volumes {
                args.push("-v");
            }
            let output = docker_compose(&args)?;
            if output.status.success() {
                output::success("Services stopped");
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                output::error(&format!("Failed to stop services: {}", stderr));
            }
        }

        DockerCommands::Restart { service } => {
            let mut args = vec!["restart"];
            if let Some(s) = &service {
                args.push(s);
            }
            let output = docker_compose(&args)?;
            if output.status.success() {
                output::success("Services restarted");
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                output::error(&format!("Failed to restart: {}", stderr));
            }
        }

        DockerCommands::Logs {
            service,
            follow,
            lines,
        } => {
            let lines_str = lines.to_string();
            let mut args = vec!["logs", "-n", &lines_str];
            if follow {
                args.push("-f");
            }
            if let Some(s) = &service {
                args.push(s);
            }
            let output = docker_compose(&args)?;
            print!("{}", String::from_utf8_lossy(&output.stdout));
            if !output.stderr.is_empty() {
                eprint!("{}", String::from_utf8_lossy(&output.stderr));
            }
        }

        DockerCommands::Ps => {
            let output = docker_compose(&["ps"])?;
            print!("{}", String::from_utf8_lossy(&output.stdout));
        }
    }

    Ok(())
}

// ── Status command ─────────────────────────────────────────────────

async fn handle_status(json: bool) -> Result<()> {
    let mut status_items = Vec::new();

    let db_ok = match alesys_core::db::create_db_pool().await {
        Ok(pool) => alesys_core::db::check_database(&pool).await,
        Err(_) => false,
    };
    status_items.push(("Database", db_ok));

    let docker_ok = std::process::Command::new("docker")
        .args(["info"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    status_items.push(("Docker", docker_ok));

    let llm_config = alesys_core::llm::LLMConfig::from_env();
    let llm_configured = !llm_config.model_path.is_empty();
    status_items.push(("LLM Config", llm_configured));

    if json {
        let data: Vec<_> = status_items
            .iter()
            .map(|(name, ok)| {
                serde_json::json!({
                    "component": name,
                    "status": if *ok { "ok" } else { "error" },
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&data)?);
    } else {
        output::header("System Status");
        for (name, ok) in &status_items {
            let indicator = if *ok {
                "\x1b[32m[ok]\x1b[0m"
            } else {
                "\x1b[31m[error]\x1b[0m"
            };
            println!("  {} {}", indicator, name);
        }
    }

    Ok(())
}

// ── Config command ─────────────────────────────────────────────────

fn handle_config(json: bool) -> Result<()> {
    let config = alesys_core::llm::LLMConfig::from_env();
    let db_url = alesys_core::db::resolve_database_url();

    if json {
        // Mask password for JSON output
        let masked = if let Some(at_idx) = db_url.find('@') {
            if let Some(colon_idx) = db_url[..at_idx].rfind(':') {
                format!("{}:***@{}", &db_url[..colon_idx + 1], &db_url[at_idx + 1..])
            } else {
                db_url.clone()
            }
        } else {
            db_url.clone()
        };

        println!(
            "{}",
            serde_json::json!({
                "database_url": masked,
                "llm": {
                    "backend": config.backend,
                    "model_path": config.model_path,
                    "context_size": config.context_size,
                    "max_tokens": config.max_tokens,
                    "temperature": config.temperature,
                },
                "api_addr": std::env::var("API_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_string()),
            })
        );
    } else {
        output::header("Configuration");

        let masked_url = if let Some(at_idx) = db_url.find('@') {
            if let Some(colon_idx) = db_url[..at_idx].rfind(':') {
                format!("{}:***@{}", &db_url[..colon_idx + 1], &db_url[at_idx + 1..])
            } else {
                db_url.clone()
            }
        } else {
            db_url.clone()
        };

        output::kv("Database URL", &masked_url);
        output::kv("LLM Backend", &config.backend);
        output::kv("LLM Model", &config.model_path);
        output::kv("Context Size", &config.context_size);
        output::kv("Max Tokens", &config.max_tokens);
        output::kv("Temperature", &config.temperature);
        output::kv(
            "API Address",
            &std::env::var("API_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_string()),
        );
    }

    Ok(())
}

// ── Graph commands ─────────────────────────────────────────────────

async fn handle_graph(action: GraphCommands, json: bool) -> Result<()> {
    let pool = alesys_core::db::create_db_pool().await?;
    let graphrag = alesys_core::GraphRAG::new(pool).await?;

    match action {
        GraphCommands::Stats => {
            let stats = graphrag.graph_stats();

            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "nodes": stats.nodes,
                        "edges": stats.edges,
                    })
                );
            } else {
                output::header("Graph Statistics");
                output::kv("Total nodes", &stats.nodes);
                output::kv("Total edges", &stats.edges);
            }
        }

        GraphCommands::Export { output } => {
            let query = alesys_core::graphrag::api::GraphQuery {
                doc_type: None,
                edge_type: None,
                depth: None,
                limit: None,
                cursor: None,
                center_node_id: None,
                include_metrics: None,
            };
            let data = graphrag.get_graph_api(&query, 0).await?;
            let json_str = serde_json::to_string_pretty(&data)?;
            std::fs::write(&output, &json_str)?;
            output::success(&format!("Graph exported to: {}", output));
        }

        GraphCommands::Path { from, to } => {
            let query = alesys_core::graphrag::api::PathQuery {
                source_id: from,
                target_id: to,
            };
            let path = graphrag.get_shortest_path(&query).await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&path)?);
            } else {
                output::header(&format!("Shortest Path: {} -> {}", from, to));
                if path.found {
                    output::kv("Distance", &path.distance);
                    output::kv("Path length", &path.path_length);
                    println!();
                    for (i, node) in path.path.iter().enumerate() {
                        if i > 0 {
                            println!("  ->");
                        }
                        println!("  {}", node);
                    }
                } else {
                    output::info("No path found");
                }
            }
        }
    }

    Ok(())
}

// ── Search command ─────────────────────────────────────────────────

async fn handle_search(query: &str, limit: usize, json: bool) -> Result<()> {
    let pool = alesys_core::db::create_db_pool().await?;

    let rows = sqlx::query(
        r#"
        SELECT f.id, d.ruta_relativa, f.contenido, ts_rank(to_tsvector('spanish', f.contenido), plainto_tsquery('spanish', $1)) as rank
        FROM fragmentos f
        JOIN documentos d ON f.documento_id = d.id
        WHERE to_tsvector('spanish', f.contenido) @@ plainto_tsquery('spanish', $1)
        ORDER BY rank DESC
        LIMIT $2
        "#,
    )
    .bind(query)
    .bind(limit as i64)
    .fetch_all(&pool)
    .await;

    match rows {
        Ok(rows) => {
            if json {
                let data: Vec<_> = rows
                    .iter()
                    .map(|r| {
                        let content: String = r.get(2);
                        serde_json::json!({
                            "fragment_id": r.get::<i32, _>(0),
                            "path": r.get::<String, _>(1),
                            "snippet": content.chars().take(200).collect::<String>(),
                            "rank": r.get::<f32, _>(3),
                        })
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&data)?);
            } else {
                if rows.is_empty() {
                    output::info("No results found");
                    return Ok(());
                }
                output::header(&format!("Search Results ({})", query));
                let mut table = output::new_table(&["ID", "Path", "Snippet", "Rank"]);
                for r in &rows {
                    let content: String = r.get(2);
                    table.add_row(vec![
                        r.get::<i32, _>(0).to_string(),
                        r.get::<String, _>(1),
                        content.chars().take(80).collect::<String>(),
                        format!("{:.4}", r.get::<f32, _>(3)),
                    ]);
                }
                println!("{}", table);
            }
        }
        Err(e) => {
            output::warn(&format!("Search requires full-text indexes. Run: alesys db migrate"));
            output::warn(&format!("Error: {}", e));
        }
    }

    Ok(())
}

// ── List command ───────────────────────────────────────────────────

async fn handle_list(json: bool) -> Result<()> {
    let pool = alesys_core::db::create_db_pool().await?;

    let rows = sqlx::query(
        "SELECT id, ruta_relativa, tipo, creado_en FROM documentos ORDER BY id LIMIT 100",
    )
    .fetch_all(&pool)
    .await?;

    if json {
        let data: Vec<_> = rows
            .iter()
            .map(|r| {
                serde_json::json!({
                    "id": r.get::<i32, _>(0),
                    "path": r.get::<String, _>(1),
                    "type": r.get::<String, _>(2),
                    "created": r.get::<chrono::NaiveDateTime, _>(3).to_string(),
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&data)?);
    } else {
        if rows.is_empty() {
            output::info("No indexed documents found");
            return Ok(());
        }
        output::header("Indexed Documents");
        let mut table = output::new_table(&["ID", "Path", "Type", "Created"]);
        for r in &rows {
            table.add_row(vec![
                r.get::<i32, _>(0).to_string(),
                r.get::<String, _>(1),
                r.get::<String, _>(2),
                r.get::<chrono::NaiveDateTime, _>(3).to_string(),
            ]);
        }
        println!("{}", table);
    }

    Ok(())
}

// ── Ask command ────────────────────────────────────────────────────

async fn handle_ask(question: &str, session_id: Option<&str>, _json: bool) -> Result<()> {
    let pool = alesys_core::db::create_db_pool().await?;
    let manager = alesys_core::SessionManager::new(pool.clone());

    let sid = match session_id {
        Some(id) => id.to_string(),
        None => manager.create_session(0, None).await?,
    };

    // Search for context using full-text search
    let rows = sqlx::query(
        r#"
        SELECT f.id, d.ruta_relativa, f.contenido
        FROM fragmentos f
        JOIN documentos d ON f.documento_id = d.id
        WHERE to_tsvector('spanish', f.contenido) @@ plainto_tsquery('spanish', $1)
        ORDER BY ts_rank(to_tsvector('spanish', f.contenido), plainto_tsquery('spanish', $1)) DESC
        LIMIT 5
        "#,
    )
    .bind(question)
    .fetch_all(&pool)
    .await?;

    if rows.is_empty() {
        output::warn("No relevant context found in knowledge base");
        output::info("Try indexing documents first: alesys db-init");
        output::info(&format!("Session created: {}", sid));
        return Ok(());
    }

    output::info(&format!("Context found: {} sources", rows.len()));
    output::info(&format!("Session: {}", sid));
    println!();

    for (i, r) in rows.iter().enumerate() {
        let content: String = r.get(2);
        println!(
            "\x1b[1m[{}]\x1b[0m {}",
            i + 1,
            r.get::<String, _>(1)
        );
        println!("    {}", content.chars().take(200).collect::<String>());
        println!();
    }

    // Check if LLM is available
    let llm_config = alesys_core::llm::LLMConfig::from_env();
    if llm_config.model_path.is_empty() {
        output::info("Configure LLM_MODEL_PATH to enable chat responses");
    } else {
        output::info("Use 'alesys llm load' to load the model, then retry for full RAG chat.");
    }

    Ok(())
}
