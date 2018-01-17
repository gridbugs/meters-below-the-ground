use append::Append;
use entity_store::*;
use direction::CardinalDirection;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Card {
    Move,
}

impl Card {
    pub fn play<A: Append<EntityChange>>(self, entity_id: EntityId, entity_store: &EntityStore,
                                         direction: CardinalDirection, changes: &mut A) {

        match self {
            Card::Move => {
                let current = entity_store.coord.get(&entity_id).unwrap();
                let delta = direction.vector();
                let new = current + delta;
                changes.append(insert::coord(entity_id, new));
            }
        }
    }
}
