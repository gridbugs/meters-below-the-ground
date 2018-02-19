use entity_store::*;
use card::*;
use animation::*;
use grid_2d::Coord;
use append::Append;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Reaction {
    TakeCard(EntityId, Card),
    RemoveEntity(EntityId),
    StartAnimation(Animation),
    EntityChange(EntityChange),
    PlayerMovedTo(Coord),
}

#[derive(Debug)]
pub struct ReactionEntityChangeAppend<'a, A: 'a + Append<Reaction>> {
    pub reactions: &'a mut A,
}

impl<'a, A: Append<Reaction>> Append<EntityChange> for ReactionEntityChangeAppend<'a, A> {
    fn append(&mut self, value: EntityChange) {
        self.reactions.append(Reaction::EntityChange(value));
    }
}
