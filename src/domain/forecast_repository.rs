use crate::domain::{forecast::Forecast, place::Place, time::Time};

pub trait ForecastRepository {
    fn find(&self, places: Vec<Place>, start_time: Time, end_time: Time) -> Vec<Forecast>;
}
