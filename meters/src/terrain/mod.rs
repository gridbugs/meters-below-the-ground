use rand::Rng;
use grid_2d::*;
use entity_store::EntityIdAllocator;
use message_queues::*;
use goal::*;

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
    ) -> Option<GoalState> {
        match &self.typ {
            &TerrainType::StaticStrings(ref strings) => {
                static_strings::populate(strings, self.config, id_allocator, messages);
                None
            }
            &TerrainType::Empty => {
                empty::populate(self.config, id_allocator, messages);
                None
            }
            &TerrainType::Dungeon => loop {
                match dungeon::populate(self.config, id_allocator, messages, rng) {
                    dungeon::DungeonPopulateResult::Retry => (),
                    dungeon::DungeonPopulateResult::NoGoalState => return None,
                    dungeon::DungeonPopulateResult::GoalState(goal_state) => {
                        return Some(goal_state)
                    }
                }
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct TerrainConfig {
    pub final_level: bool,
    pub goal_type: GoalType,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerrainInfo {
    pub typ: TerrainType,
    pub config: TerrainConfig,
}
