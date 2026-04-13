use anyhow::Result;

use crate::{infrastructure::forecast_args::Args, interface::prompt};

pub mod application;
pub mod bootstrap;
pub mod domain;
pub mod infrastructure;
pub mod interface;
pub mod logging;

pub async fn run(args: &Args) -> Result<()> {
    let get_risk_summary_use_case = bootstrap::bootstrap();

    let risk_summary = get_risk_summary_use_case.execute(
        args.places.iter().map(|p| p.into()).collect(),
        args.start_time.clone().into(),
        args.end_time.clone().into(),
    );

    prompt::run_prompt(&risk_summary).await?;

    Ok(())
}
