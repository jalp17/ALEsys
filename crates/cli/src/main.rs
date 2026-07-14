//! ALEsys CLI - Interfaz de línea de comandos

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "alesys")]
#[command(about = "GraphRAG-PG: PostgreSQL Graph & Vector Ingestion Engine")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Verbose output
    #[arg(short, long, default_value_t = false)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Inicializar base de datos
    DbInit,

    /// Eliminar base de datos
    DbDrop {
        #[arg(long, default_value_t = false)]
        force: bool,
    },

    /// Ejecutar pipeline de ingesta
    Run {
        #[arg(short, long, default_value = "./books")]
        input: String,

        #[arg(long, default_value_t = 1000)]
        chunk_size: usize,

        #[arg(long, default_value_t = 200)]
        chunk_overlap: usize,

        /// Dry run (no inserta en DB)
        #[arg(long, default_value_t = false)]
        dry_run: bool,
    },

    /// Búsqueda híbrida
    Query {
        query: String,

        #[arg(short, long, default_value = "hybrid")]
        mode: String, // vector, graph, hybrid
    },

    /// Chat con contexto RAG
    Ask {
        question: String,

        #[arg(short, long)]
        session: Option<String>,
    },

    /// Listar documentos indexados
    List,

    /// Gestionar sesiones
    Session {
        #[command(subcommand)]
        action: SessionCommands,
    },
}

#[derive(Subcommand)]
enum SessionCommands {
    /// Crear nueva sesión
    New {
        #[arg(short, long)]
        name: Option<String>,
    },

    /// Listar sesiones activas
    List,

    /// Cerrar sesión
    Close { session_id: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Setup logging
    if cli.verbose {
        std::env::set_var("RUST_LOG", "debug");
    }

    tracing_subscriber::fmt::init();

    // Load environment
    dotenvy::dotenv().ok();

    match cli.command {
        Commands::DbInit => {
            println!("🗄️  Inicializando base de datos...");
            // TODO: Implementar
            println!("✅ DB inicializada");
        }

        Commands::DbDrop { force } => {
            if !force {
                print!("⚠️  ¿Seguro que deseas eliminar la DB? [y/N]: ");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;

                if input.trim().to_lowercase() != "y" {
                    println!("❌ Cancelado");
                    return Ok(());
                }
            }

            println!("🗑️  Eliminando base de datos...");
            // TODO: Implementar
            println!("✅ DB eliminada");
        }

        Commands::Run {
            input,
            chunk_size,
            chunk_overlap,
            dry_run,
        } => {
            println!("🚀 Ejecutando pipeline...");
            println!("   Input: {}", input);
            println!("   Chunk size: {}", chunk_size);
            println!("   Overlap: {}", chunk_overlap);
            println!("   Dry run: {}", dry_run);

            // TODO: Implementar pipeline de ingesta
            println!("✅ Pipeline completado");
        }

        Commands::Query { query, mode } => {
            println!("🔍 Buscando: {}", query);
            println!("   Modo: {}", mode);

            // TODO: Implementar búsqueda
            println!("❌ Implementar en Fase 1");
        }

        Commands::Ask { question, session } => {
            println!("💬 Pregunta: {}", question);
            if let Some(sid) = session {
                println!("   Sesión: {}", sid);
            }

            // TODO: Implementar chat con RAG
            println!("❌ Implementar en Fase 1");
        }

        Commands::List => {
            println!("📄 Documentos indexados:");
            // TODO: Implementar
            println!("❌ Implementar en Fase 1");
        }

        Commands::Session { action } => {
            match action {
                SessionCommands::New { name } => {
                    println!("🆕 Creando sesión...");
                    if let Some(n) = name {
                        println!("   Nombre: {}", n);
                    }
                    // TODO: Implementar
                }
                SessionCommands::List => {
                    println!("📋 Sesiones activas:");
                    // TODO: Implementar
                }
                SessionCommands::Close { session_id } => {
                    println!("🔒 Cerrando sesión: {}", session_id);
                    // TODO: Implementar
                }
            }
        }
    }

    Ok(())
}
