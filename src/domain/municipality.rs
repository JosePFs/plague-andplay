use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub struct Municipality(pub String);

impl Municipality {
    pub fn new(value: String) -> Self {
        Self(value)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for Municipality {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for Municipality {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl Display for Municipality {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
