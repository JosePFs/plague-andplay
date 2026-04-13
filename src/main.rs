use anyhow::Result;
use clap::Parser as _;

use plague_andplay::{infrastructure::forecast_args::Args, logging::init_logging};

#[tokio::main]
async fn main() -> Result<()> {
    init_logging();

    let args = Args::parse();

    plague_andplay::run(&args).await?;

    Ok(())
}
