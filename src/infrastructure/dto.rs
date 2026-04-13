use std::{collections::HashMap, fmt::Display};

use crate::domain::plague::Plague as DomainPlague;
use crate::domain::plague::PlagueMetadata as DomainPlagueMetadata;
use crate::domain::plague::PlagueType as DomainPlagueType;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RawForecast {
    pub places: Vec<RawPlace>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub enum RawPlaceStatus {
    Found,
    LocationNotFound,
    ForecastInfoNotFound,
}

impl RawPlaceStatus {
    pub fn is_not_found(&self) -> bool {
        matches!(
            self,
            RawPlaceStatus::LocationNotFound | RawPlaceStatus::ForecastInfoNotFound
        )
    }
}

impl TryFrom<String> for RawPlaceStatus {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "Found" => Ok(RawPlaceStatus::Found),
            "LocationNotFound" => Ok(RawPlaceStatus::LocationNotFound),
            "ForecastInfoNotFound" => Ok(RawPlaceStatus::ForecastInfoNotFound),
            _ => Err(format!("Invalid status: '{}'", s)),
        }
    }
}

impl Display for RawPlaceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RawPlaceStatus::Found => write!(f, "Found"),
            RawPlaceStatus::LocationNotFound => write!(f, "Location not found"),
            RawPlaceStatus::ForecastInfoNotFound => write!(f, "Forecast info not found"),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RawPlace {
    pub name: String,
    pub municipality: String,
    pub days: Vec<RawDay>,
    pub status: RawPlaceStatus,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Hash)]
pub enum RawDayParameter {
    #[serde(rename = "temperature")]
    Temperature,
    #[serde(rename = "relative_humidity")]
    RelativeHumidity,
    #[serde(rename = "precipitation_amount")]
    PrecipitationAmount,
    #[serde(rename = "wind")]
    Wind,
}

#[derive(Debug, Deserialize)]
pub struct RawDay {
    pub date: String,
    pub values: HashMap<RawDayParameter, Vec<RawValue>>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum WindOrScalar {
    Wind { speed: f64, direction: f64 },
    Scalar(f64),
}

#[derive(Debug, Deserialize)]
pub struct RawValue {
    pub time: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct PlagueMetadata {
    pub min_temp: f64,
    pub max_temp: f64,
    pub min_humidity: f64,
    pub min_precipitation: f64,
    #[serde(default)]
    pub max_precipitation: Option<f64>,
    pub r#type: PlagueType,
}

impl PlagueMetadata {
    pub fn new(
        min_temp: f64,
        max_temp: f64,
        min_humidity: f64,
        min_precipitation: f64,
        max_precipitation: Option<f64>,
        r#type: PlagueType,
    ) -> Self {
        Self {
            min_temp,
            max_temp,
            min_humidity,
            min_precipitation,
            max_precipitation,
            r#type,
        }
    }
}

impl From<PlagueMetadata> for DomainPlagueMetadata {
    fn from(metadata: PlagueMetadata) -> Self {
        Self {
            min_temp: metadata.min_temp,
            max_temp: metadata.max_temp,
            min_humidity: metadata.min_humidity,
            min_precipitation: metadata.min_precipitation,
            max_precipitation: metadata.max_precipitation,
            r#type: metadata.r#type.into(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub enum PlagueType {
    Pest,
    Disease,
}

impl PlagueType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "pest" => Self::Pest,
            "disease" => Self::Disease,
            _ => panic!("Invalid plague type: {}", s),
        }
    }
}

impl From<PlagueType> for DomainPlagueType {
    fn from(r#type: PlagueType) -> Self {
        match r#type {
            PlagueType::Pest => Self::Pest,
            PlagueType::Disease => Self::Disease,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Plague {
    pub id: String,
    pub text: String,
    pub metadata: PlagueMetadata,
}

impl Plague {
    pub fn new(id: String, text: String, metadata: PlagueMetadata) -> Self {
        Self { id, text, metadata }
    }
}

impl From<Plague> for DomainPlague {
    fn from(plague: Plague) -> Self {
        Self {
            id: plague.id,
            text: plague.text,
            metadata: plague.metadata.into(),
        }
    }
}
