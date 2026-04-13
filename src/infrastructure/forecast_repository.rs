use crate::{
    domain::{
        forecast::{Forecast, Summary},
        forecast_repository::ForecastRepository as ForecastRepositoryTrait,
        place::Place,
        time::Time,
    },
    infrastructure::{
        command_runner,
        dto::{RawDayParameter, RawForecast, WindOrScalar},
    },
};

pub struct ForecastRepository;

impl ForecastRepository {
    pub fn new() -> Self {
        Self
    }
}

impl ForecastRepositoryTrait for ForecastRepository {
    fn find(&self, places: Vec<Place>, start_time: Time, end_time: Time) -> Vec<Forecast> {
        let raw_string = match command_runner::run_forecast(
            places.into_iter().map(|p| p.into()).collect(),
            start_time,
            end_time,
        ) {
            Ok(raw_string) => raw_string,
            Err(e) => {
                tracing::error!("Error running forecast command: {:?}", e);
                return vec![];
            }
        };

        let raw_forecasts: Vec<RawForecast> = serde_json::from_str(&raw_string).unwrap_or_default();

        raw_forecasts
            .into_iter()
            .flat_map(|raw_forecast| {
                raw_forecast
                    .places
                    .iter()
                    .map(|place| {
                        let mut summary = Summary::default();

                        if place.status.is_not_found() {
                            return Forecast {
                                place: place.name.clone(),
                                municipality: place.municipality.clone(),
                                summary,
                                errors: vec![place.status.to_string()],
                            };
                        }

                        place.days.iter().for_each(|day| {
                            if let Some(values) = day.values.get(&RawDayParameter::Temperature) {
                                for value in values {
                                    let v = value.value.as_f64().unwrap_or(0.0);
                                    summary.add_temperature(
                                        day.date.clone(),
                                        value.time.clone(),
                                        v,
                                    );
                                }
                            }

                            if let Some(values) = day.values.get(&RawDayParameter::RelativeHumidity)
                            {
                                for value in values {
                                    let v = value.value.as_f64().unwrap_or(0.0);
                                    summary.add_relative_humidity(
                                        day.date.clone(),
                                        value.time.clone(),
                                        v,
                                    );
                                }
                            }

                            if let Some(values) =
                                day.values.get(&RawDayParameter::PrecipitationAmount)
                            {
                                for value in values {
                                    let v = value.value.as_f64().unwrap_or(0.0);
                                    summary.add_precipitation_amount(
                                        day.date.clone(),
                                        value.time.clone(),
                                        v,
                                    );
                                }
                            }

                            if let Some(values) = day.values.get(&RawDayParameter::Wind) {
                                for value in values {
                                    match serde_json::from_value::<WindOrScalar>(
                                        value.value.clone(),
                                    ) {
                                        Ok(WindOrScalar::Wind { speed, direction }) => {
                                            summary.add_wind(
                                                day.date.clone(),
                                                value.time.clone(),
                                                speed,
                                                direction,
                                            );
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        });

                        Forecast {
                            place: place.name.clone(),
                            municipality: place.municipality.clone(),
                            summary,
                            errors: vec![],
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    }
}
