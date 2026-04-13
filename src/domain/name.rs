use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub struct Name(pub String);

impl From<&str> for Name {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<String> for Name {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl Name {
    pub fn from_str(value: &str) -> Self {
        Self(value.to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn to_lowercase(&self) -> String {
        self.0.to_lowercase()
    }
}

impl Display for Name {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
