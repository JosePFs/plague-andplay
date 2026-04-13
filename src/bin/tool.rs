use clap::{Parser, Subcommand};

use anyhow::Result;
use plague_andplay::logging::init_logging;

#[derive(Parser)]
#[command(name = "tool", about = "CLI auxiliar", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Generate,
    Clean,
}

fn main() -> Result<()> {
    init_logging();

    let cli = Cli::parse();

    match cli.command {
        Commands::Generate => {
            tracing::info!("Generating...");

            let current_dir = std::env::current_dir()?;
            let documents_dir = current_dir.join("documents");

            let pdf_files = documents_dir.join("pdf");
            let esae_pfg_cas_path = pdf_files.join("plagues_and_diseases.pdf");
            let esae_pfg_cas_content = pdf_extract::extract_text(&esae_pfg_cas_path)?;

            let tmp_dir = current_dir.join("tmp");
            let tmp_file = tmp_dir.join("plagues_and_diseases.txt");

            std::fs::write(tmp_file, esae_pfg_cas_content)?;
        }
        Commands::Clean => {
            tracing::info!("Cleaning...");

            let current_dir = std::env::current_dir()?;
            let tmp_dir = current_dir.join("tmp");
            let tmp_file = tmp_dir.join("plagues_and_diseases.txt");

            std::fs::remove_file(tmp_file)?;
        }
    }

    Ok(())
}
