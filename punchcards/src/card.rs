use entity_store::*;
use direction::CardinalDirection;
use common_animations;
use message_queues::PushMessages;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Card {
    Move,
    Punch,
}

impl Card {
    pub fn play<M>(
        self,
        entity_id: EntityId,
        entity_store: &EntityStore,
        direction: CardinalDirection,
        id_allocator: &mut EntityIdAllocator,
        messages: &mut M,
    ) where
        M: PushMessages,
    {
        match self {
            Card::Move => {
                let current = entity_store.coord.get(&entity_id).unwrap();
                let delta = direction.coord();
                let new = *current + delta;
                messages.change(insert::coord(entity_id, new));
            }
            Card::Punch => {
                let source_coord = entity_store.coord.get(&entity_id).unwrap();
                let delta = direction.vector();
                let coord = *source_coord + delta;
                let punch_id = id_allocator.allocate();

                common_animations::punch(punch_id, coord, direction, messages);
            }
        }
    }
}
