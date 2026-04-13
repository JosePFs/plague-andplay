use std::process::Command;

use anyhow::Result;

use crate::{domain::time::Time, infrastructure::forecast_args::PlacePair};

pub fn run_forecast(places: Vec<PlacePair>, start_time: Time, end_time: Time) -> Result<String> {
    let mut cmd = Command::new("forecast");

    cmd.arg("forecast");

    for place in places {
        cmd.arg("--places").arg(place.to_string());
    }

    cmd.arg("--start-time").arg(start_time.to_string());
    cmd.arg("--end-time").arg(end_time.to_string());

    if let Ok(key) = std::env::var("API_KEY") {
        cmd.env("API_KEY", key);
    }

    if let Ok(url) = std::env::var("BASE_URL") {
        cmd.env("BASE_URL", url);
    }

    let output = cmd.output()?;

    Ok(String::from_utf8(output.stdout)?)
}
