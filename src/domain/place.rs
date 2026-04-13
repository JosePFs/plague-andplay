use std::fmt::{Display, Formatter};

use crate::domain::{municipality::Municipality, name::Name};

#[derive(Debug, Clone)]
pub struct Place {
    pub name: Name,
    pub municipality: Municipality,
}

impl Place {
    pub fn new(name: Name, municipality: Municipality) -> Self {
        Self { name, municipality }
    }
}

impl Display for Place {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
