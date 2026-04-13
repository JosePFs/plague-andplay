use crate::domain::plague::Plague;

pub trait PlagueRepository {
    fn find_all(&self) -> Vec<Plague>;
}
