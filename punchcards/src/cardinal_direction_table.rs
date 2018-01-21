use std::mem;
use direction::CardinalDirection;

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

    pub fn take(&mut self, direction: CardinalDirection) -> Option<T> {
        let field = match direction {
            CardinalDirection::North => &mut self.north,
            CardinalDirection::East => &mut self.east,
            CardinalDirection::South => &mut self.south,
            CardinalDirection::West => &mut self.west,
        };
        field.take()
    }

    pub fn get(&self, direction: CardinalDirection) -> Option<&T> {
        let field = match direction {
            CardinalDirection::North => &self.north,
            CardinalDirection::East => &self.east,
            CardinalDirection::South => &self.south,
            CardinalDirection::West => &self.west,
        };
        field.as_ref()
    }

    pub fn contains(&self, direction: CardinalDirection) -> bool {
        let field = match direction {
            CardinalDirection::North => &self.north,
            CardinalDirection::East => &self.east,
            CardinalDirection::South => &self.south,
            CardinalDirection::West => &self.west,
        };
        field.is_some()
    }

    pub fn insert(&mut self, direction: CardinalDirection, data: T) -> Option<T> {
         let field = match direction {
            CardinalDirection::North => &mut self.north,
            CardinalDirection::East => &mut self.east,
            CardinalDirection::South => &mut self.south,
            CardinalDirection::West => &mut self.west,
        };
        mem::replace(field, Some(data))
    }
}
