use crate::application::use_case::get_risk_summary::GetRiskSummaryUseCase;
use crate::infrastructure::forecast_repository::ForecastRepository;
use crate::infrastructure::plague_repository::PlagueRepository;

pub fn bootstrap() -> GetRiskSummaryUseCase<ForecastRepository, PlagueRepository> {
    GetRiskSummaryUseCase::new(ForecastRepository::new(), PlagueRepository::new())
}
