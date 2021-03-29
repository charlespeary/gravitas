use lasso::{Rodeo, Spur};

pub type RodeoInterner = Rodeo<Spur>;

pub trait Interner {
    fn get_or_intern<K>(&mut self, str: &str) -> K
    where
        Self: Sized;
}
