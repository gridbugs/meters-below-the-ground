use grid_2d::*;
use entity_store::EntityIdAllocator;
use message_queues::*;

mod static_strings;

pub enum TerrainType {
    StaticStrings(Vec<&'static str>),
}

impl TerrainType {
    pub fn size(&self) -> Size {
        match self {
            &TerrainType::StaticStrings(ref strings) => {
                Size::new(strings[0].len() as u32, strings.len() as u32)
            }
        }
    }

    pub fn populate(
        &self,
        id_allocator: &mut EntityIdAllocator,
        messages: &mut MessageQueues,
    ) {
        match self {
            &TerrainType::StaticStrings(ref strings) => {
                static_strings::populate(strings, id_allocator, messages);
            }
        }
    }
}
