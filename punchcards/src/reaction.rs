use entity_store::*;
use card::*;
use animation::*;
use grid_2d::Coord;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Reaction {
    TakeCard(EntityId, Card),
    RemoveEntity(EntityId),
    StartAnimation(Animation),
    EntityChange(EntityChange),
    PlayerMovedTo(Coord),
}

impl From<EntityChange> for Reaction {
    fn from(entity_change: EntityChange) -> Self {
        Reaction::EntityChange(entity_change)
    }
}
