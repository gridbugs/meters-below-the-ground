use entity_store::*;
use direction::CardinalDirection;
use common_animations;
use message_queues::PushMessages;
use prototypes;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Card {
    Move,
    Punch,
    Shoot,
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
            Card::Shoot => {
                let entity_coord = entity_store.coord.get(&entity_id).cloned().unwrap();
                let start_coord = entity_coord + direction.coord();
                let bullet_id = id_allocator.allocate();
                prototypes::bullet(bullet_id, start_coord, direction, messages);
                common_animations::bullet(bullet_id, messages);
            }
        }
    }
}
