use crate::domain::{plague::Plague, plague_repository::PlagueRepository as PlagueRepositoryTrait};
use crate::infrastructure::dto::Plague as DtoPlague;

pub struct PlagueRepository;

impl PlagueRepository {
    pub fn new() -> Self {
        Self
    }
}

impl PlagueRepositoryTrait for PlagueRepository {
    fn find_all(&self) -> Vec<Plague> {
        let Ok(current_dir) = std::env::current_dir() else {
            return vec![];
        };
        let documents_dir = current_dir.join("documents");
        let json_dir = documents_dir.join("json");

        json_dir
            .read_dir()
            .unwrap()
            .map(|entry| {
                let entry = entry.unwrap();
                let path = entry.path();
                let content = std::fs::read_to_string(path).unwrap();
                let plague: DtoPlague = serde_json::from_str(&content).unwrap();
                plague
            })
            .map(|dto_plague| dto_plague.into())
            .collect::<Vec<Plague>>()
    }
}
