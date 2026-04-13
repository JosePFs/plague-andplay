use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use clap::{ArgAction, Parser};

use crate::domain::place::Place;
use crate::domain::time::Time as TimeDomain;

#[derive(Debug, Clone)]
pub struct PlacePair {
    pub location: String,
    pub municipality: String,
}

impl FromStr for PlacePair {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.splitn(2, '/');

        Ok(PlacePair {
            location: parts.next().ok_or("missing location")?.to_string(),
            municipality: parts.next().ok_or("missing municipality")?.to_string(),
        })
    }
}

impl Display for PlacePair {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.location, self.municipality)
    }
}

impl From<Place> for PlacePair {
    fn from(place: Place) -> Self {
        Self {
            location: place.name.to_string(),
            municipality: place.municipality.to_string(),
        }
    }
}

impl From<&PlacePair> for Place {
    fn from(place: &PlacePair) -> Self {
        Self {
            name: place.location.clone().into(),
            municipality: place.municipality.clone().into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Time(pub String);

impl From<&str> for Time {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<String> for Time {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl Display for Time {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Time> for TimeDomain {
    fn from(value: Time) -> Self {
        Self(value.0)
    }
}

#[derive(Parser, Debug)]
#[command(about = "Get forecast information")]
pub struct Args {
    #[arg(short, long, required = true, num_args = 1.., action = ArgAction::Append, value_name = "place/municipality")]
    pub places: Vec<PlacePair>,

    #[arg(short, long, required = true)]
    pub start_time: Time,

    #[arg(short, long, required = true)]
    pub end_time: Time,
}
