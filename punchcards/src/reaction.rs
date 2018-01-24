use entity_store::*;
use card::*;
use animation::*;

pub enum Reaction {
    TakeCard(EntityId, Card),
    RemoveEntity(EntityId),
    StartAnimation(Animation),
    EntityChange(EntityChange),
}
