use rand::Rng;
use grid_2d::*;
use entity_store::EntityIdAllocator;
use message_queues::*;

mod dungeon;
mod static_strings;
mod empty;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TerrainType {
    StaticStrings(Vec<String>),
    Empty,
    Dungeon,
}

impl TerrainInfo {
    pub fn size(&self) -> Size {
        match &self.typ {
            &TerrainType::StaticStrings(ref strings) => {
                Size::new(strings[0].len() as u32, strings.len() as u32)
            }
            &TerrainType::Empty => empty::size(),
            &TerrainType::Dungeon => dungeon::size(),
        }
    }

    pub fn populate<R: Rng>(
        &self,
        id_allocator: &mut EntityIdAllocator,
        messages: &mut MessageQueues,
        rng: &mut R,
    ) {
        match &self.typ {
            &TerrainType::StaticStrings(ref strings) => {
                static_strings::populate(strings, self.config, id_allocator, messages);
            }
            &TerrainType::Empty => {
                empty::populate(self.config, id_allocator, messages);
            }
            &TerrainType::Dungeon => {
                while !dungeon::populate(self.config, id_allocator, messages, rng) {}
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct TerrainConfig {
    pub final_level: bool,
}

impl Default for TerrainConfig {
    fn default() -> Self {
        Self { final_level: false }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerrainInfo {
    pub typ: TerrainType,
    pub config: TerrainConfig,
}
