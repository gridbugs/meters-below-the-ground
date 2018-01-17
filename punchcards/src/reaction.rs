use entity_store::*;
use card::*;

pub enum Reaction {
    TakeCard(EntityId, Card),
}
