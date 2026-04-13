use std::collections::HashMap;

use crate::domain::{place::Place, plague::PlagueId};

#[derive(Debug, Clone)]
pub struct DailyForecastRiskSummary {
    pub average_temperature: f64,
    pub average_relative_humidity: f64,
    pub precipitation_amount_accumulated: f64,
    pub risk_plagues: Vec<PlagueId>,
}

impl DailyForecastRiskSummary {
    pub fn new(
        average_temperature: f64,
        average_relative_humidity: f64,
        precipitation_amount_accumulated: f64,
        risk_plagues: Vec<PlagueId>,
    ) -> Self {
        Self {
            average_temperature,
            average_relative_humidity,
            precipitation_amount_accumulated,
            risk_plagues,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlaceForecastRiskSummary {
    pub place: Place,
    pub daily_risk_summary: HashMap<String, DailyForecastRiskSummary>,
}

impl PlaceForecastRiskSummary {
    pub fn new(
        place: Place,
        daily_risk_summary: HashMap<String, DailyForecastRiskSummary>,
    ) -> Self {
        Self {
            place,
            daily_risk_summary,
        }
    }
}
