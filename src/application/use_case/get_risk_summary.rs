use std::collections::HashMap;

use crate::domain::{
    forecast_repository::ForecastRepository,
    place::Place,
    place_forecast_summary::{DailyForecastRiskSummary, PlaceForecastRiskSummary},
    plague::Plague,
    plague_repository::PlagueRepository,
    time::Time,
};

#[derive(Debug, Clone)]
pub struct GetRiskSummaryUseCaseResult {
    place_forecast_risk_summaries: Vec<PlaceForecastRiskSummary>,
    risk_plagues: Vec<Plague>,
}

impl GetRiskSummaryUseCaseResult {
    pub fn new(
        place_forecast_risk_summaries: Vec<PlaceForecastRiskSummary>,
        risk_plagues: Vec<Plague>,
    ) -> Self {
        Self {
            place_forecast_risk_summaries,
            risk_plagues,
        }
    }

    pub fn place_forecast_risk_summaries(&self) -> &Vec<PlaceForecastRiskSummary> {
        &self.place_forecast_risk_summaries
    }

    pub fn risk_plagues(&self) -> &Vec<Plague> {
        &self.risk_plagues
    }
}

pub struct GetRiskSummaryUseCase<T: ForecastRepository, U: PlagueRepository> {
    forecast_repository: T,
    plague_repository: U,
}

impl<T: ForecastRepository, U: PlagueRepository> GetRiskSummaryUseCase<T, U> {
    pub fn new(forecast_repository: T, plague_repository: U) -> Self {
        Self {
            forecast_repository,
            plague_repository,
        }
    }
}

impl<T: ForecastRepository, U: PlagueRepository> GetRiskSummaryUseCase<T, U> {
    pub fn execute(
        &self,
        places: Vec<Place>,
        start_time: Time,
        end_time: Time,
    ) -> GetRiskSummaryUseCaseResult {
        let plagues = self.plague_repository.find_all();
        let forecasts = self.forecast_repository.find(places, start_time, end_time);

        let place_risk_summaries = forecasts
            .iter()
            .map(|forecast| {
                let place = forecast.place.clone();
                let municipality = forecast.municipality.clone();
                let place = Place::new(place.into(), municipality.into());
                let daily_report = forecast.summary.get_daily_report();
                let daily_risk_summary = daily_report
                    .iter()
                    .map(|(date, report)| {
                        (
                            date.clone(),
                            DailyForecastRiskSummary::new(
                                report.mean_temperature,
                                report.mean_relative_humidity,
                                report.precipitation_amount_accumulated,
                                plagues
                                    .iter()
                                    .filter(|plague| {
                                        plague.is_risky(
                                            report.mean_temperature,
                                            report.mean_relative_humidity,
                                            report.precipitation_amount_accumulated,
                                        )
                                    })
                                    .map(|plague| plague.id.clone().into())
                                    .collect(),
                            ),
                        )
                    })
                    .collect::<HashMap<String, DailyForecastRiskSummary>>();
                PlaceForecastRiskSummary::new(place, daily_risk_summary)
            })
            .collect::<Vec<PlaceForecastRiskSummary>>();

        let risk_plagues = plagues
            .iter()
            .filter(|plague| {
                place_risk_summaries
                    .iter()
                    .any(|place_forecast_risk_summary| {
                        place_forecast_risk_summary.daily_risk_summary.iter().any(
                            |(_, daily_risk_summary)| {
                                daily_risk_summary
                                    .risk_plagues
                                    .contains(&plague.id.clone().into())
                            },
                        )
                    })
            })
            .map(|plague| plague.clone())
            .collect::<Vec<Plague>>();

        GetRiskSummaryUseCaseResult::new(place_risk_summaries, risk_plagues)
    }
}
