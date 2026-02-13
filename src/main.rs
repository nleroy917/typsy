use clap::{CommandFactory, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "typsy")]
#[command(version, about = "Static site generator for Typst -> HTML")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Build the static site into the `out/` directory.
    Build {
        /// Root directory of the project (defaults to auto-detected)
        #[arg(short, long)]
        root: Option<PathBuf>,
    },
    /// Start a local development server with live reloading.
    Dev {
        /// Port for local server
        #[arg(short, long, default_value = "3000")]
        port: u16,
    },
    /// Initialize a new typsy project in the specified directory.
    Init {
        /// Directory to initialize (defaults to current directory)
        #[arg(short, long)]
        dir: Option<PathBuf>,
    },
}

fn main() {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .init();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Build { root }) => {
            let root = root.unwrap_or_else(|| {
                typsy::build::find_root().unwrap_or_else(|e| {
                    tracing::error!("{e}");
                    std::process::exit(1);
                })
            });
            let report = typsy::build::build(&root, false);
            if !report.failures.is_empty() {
                std::process::exit(1);
            }
        }
        Some(Commands::Dev { port }) => {
            let root = typsy::build::find_root().unwrap_or_else(|e| {
                tracing::error!("{e}");
                std::process::exit(1);
            });
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                if let Err(e) = typsy::server::run_dev_server(root, port).await {
                    tracing::error!("dev server error: {e}");
                    std::process::exit(1);
                }
            });
        }
        Some(Commands::Init { dir }) => {
            let project_dir = dir.unwrap_or_else(|| std::env::current_dir().unwrap());
            if let Err(e) = typsy::init::init_new_typsy_project(&project_dir) {
                tracing::error!("project initialization error: {e}");
                std::process::exit(1);
            }
        }
        None => {
            Cli::command().print_help().unwrap();
        }
    }
}
