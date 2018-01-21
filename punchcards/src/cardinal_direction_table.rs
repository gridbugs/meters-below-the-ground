use std::mem;
use direction::CardinalDirection;
use self::CardinalDirection::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardinalDirectionTable<T> {
    pub north: Option<T>,
    pub east: Option<T>,
    pub south: Option<T>,
    pub west: Option<T>,
}

impl<T> CardinalDirectionTable<T> {
    pub fn new() -> Self {
        CardinalDirectionTable {
            north: None,
            east: None,
            south: None,
            west: None,
        }
    }

    pub fn is_full(&self) -> bool {
        self.north.is_some() &&
            self.east.is_some() &&
            self.south.is_some() &&
            self.west.is_some()
    }

    pub fn take(&mut self, direction: CardinalDirection) -> Option<T> {
        let field = match direction {
            North => &mut self.north,
            East => &mut self.east,
            South => &mut self.south,
            West => &mut self.west,
        };
        field.take()
    }

    pub fn get(&self, direction: CardinalDirection) -> Option<&T> {
        let field = match direction {
            North => &self.north,
            East => &self.east,
            South => &self.south,
            West => &self.west,
        };
        field.as_ref()
    }

    pub fn contains(&self, direction: CardinalDirection) -> bool {
        let field = match direction {
            North => &self.north,
            East => &self.east,
            South => &self.south,
            West => &self.west,
        };
        field.is_some()
    }

    pub fn insert(&mut self, direction: CardinalDirection, data: T) -> Option<T> {
         let field = match direction {
            North => &mut self.north,
            East => &mut self.east,
            South => &mut self.south,
            West => &mut self.west,
        };
        mem::replace(field, Some(data))
    }
}
