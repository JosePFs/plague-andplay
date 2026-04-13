use std::fmt::{Display, Formatter};

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
