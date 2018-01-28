use entity_store::*;
use card::*;
use animation::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Reaction {
    TakeCard(EntityId, Card),
    RemoveEntity(EntityId),
    StartAnimation(Animation),
    EntityChange(EntityChange),
}
