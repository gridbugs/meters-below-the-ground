use grid_2d::*;
use entity_store::EntityIdAllocator;
use message_queues::*;
use prototypes;
use card::Card;
use tile::Tile;

pub enum TerrainType {
    FromStrings(Vec<&'static str>),
}

impl TerrainType {
    pub fn size(&self) -> Size {
        match self {
            &TerrainType::FromStrings(ref strings) => {
                Size::new(strings[0].len() as u32, strings.len() as u32)
            }
        }
    }

    pub fn populate(&self, id_allocator: &mut EntityIdAllocator, messages: &mut MessageQueues) {
        match self {
            &TerrainType::FromStrings(ref strings) => {
                Self::populate_from_strings(strings, id_allocator, messages);
            }
        }
    }

    fn populate_from_strings(strings: &Vec<&'static str>, id_allocator: &mut EntityIdAllocator, messages: &mut MessageQueues) {
        for (y, line) in strings.iter().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                let coord = Coord::new(x as i32, y as i32);
                match ch {
                    '#' => {
                        prototypes::wall(id_allocator.allocate(), coord, messages);
                        prototypes::floor(id_allocator.allocate(), coord, messages);
                    }
                    '.' => {
                        prototypes::floor(id_allocator.allocate(), coord, messages);
                    }
                    'm' => {
                        prototypes::card(
                            id_allocator.allocate(),
                            coord,
                            Card::Move,
                            Tile::CardMove,
                            messages,
                            );
                        prototypes::floor(id_allocator.allocate(), coord, messages);
                    }
                    '0' => {
                        prototypes::target_dummy(id_allocator.allocate(), coord, messages);
                        prototypes::floor(id_allocator.allocate(), coord, messages);
                    }
                    '1' => {
                        prototypes::small_robot(id_allocator.allocate(), coord, messages);
                        prototypes::floor(id_allocator.allocate(), coord, messages);
                    }
                    '@' => {
                        let id = id_allocator.allocate();
                        prototypes::player(id, coord, messages);
                        prototypes::floor(id_allocator.allocate(), coord, messages);
                    }
                    _ => panic!("unexpected character"),
                }
            }
        }
    }
}
