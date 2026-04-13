#[derive(Debug, Clone)]
pub struct PlagueMetadata {
    pub min_temp: f64,
    pub max_temp: f64,
    pub min_humidity: f64,
    pub min_precipitation: f64,
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct PlagueId(String);

impl PlagueId {
    pub fn new(id: String) -> Self {
        Self(id)
    }
}

impl From<String> for PlagueId {
    fn from(id: String) -> Self {
        Self(id)
    }
}

impl From<PlagueId> for String {
    fn from(id: PlagueId) -> Self {
        id.0
    }
}

#[derive(Debug, Clone)]
pub struct Plague {
    pub id: String,
    pub text: String,
    pub metadata: PlagueMetadata,
}

impl Plague {
    pub fn new(id: String, text: String, metadata: PlagueMetadata) -> Self {
        Self { id, text, metadata }
    }

    pub fn is_risky(
        &self,
        temperature: f64,
        relative_humidity: f64,
        precipitation_daily_mm: f64,
    ) -> bool {
        let temp_ok =
            temperature >= self.metadata.min_temp && temperature <= self.metadata.max_temp;
        let humidity_ok = relative_humidity >= self.metadata.min_humidity;
        if !temp_ok || !humidity_ok {
            return false;
        }
        match self.metadata.r#type {
            PlagueType::Disease => {
                self.metadata.min_precipitation == 0.0
                    || precipitation_daily_mm >= self.metadata.min_precipitation
            }
            PlagueType::Pest => match self.metadata.max_precipitation {
                Some(max) => precipitation_daily_mm <= max,
                None => true,
            },
        }
    }
}
