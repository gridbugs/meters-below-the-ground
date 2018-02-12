use std::cmp::Ordering;

#[derive(PartialEq, Eq, Debug)]
pub struct InvertOrd<T: Ord>(T);

impl<T: Ord> PartialOrd for InvertOrd<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(other.0.cmp(&self.0))
    }
}

impl<T: Ord> Ord for InvertOrd<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.0.cmp(&self.0)
    }
}

impl<T: Ord> InvertOrd<T> {
    pub fn new(t: T) -> Self { InvertOrd(t) }
    pub fn into_inner(self) -> T { self.0 }
}
