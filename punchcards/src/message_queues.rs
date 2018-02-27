use std::mem;
use entity_store::*;
use animation::*;
use card::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DoubleBuffer<T> {
    pub current: Vec<T>,
    pub swap: Vec<T>,
}

impl<T> DoubleBuffer<T> {
    fn new() -> Self {
        Self {
            current: Vec::new(),
            swap: Vec::new(),
        }
    }

    pub fn swap(&mut self) {
        mem::swap(&mut self.current, &mut self.swap);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MessageQueues {
    pub animations: DoubleBuffer<Animation>,
    pub changes: DoubleBuffer<EntityChange>,
    pub removed_entities: Vec<EntityId>,
    pub take_cards: Vec<(EntityId, Card)>,
    pub player_moved_to: Option<Coord>,
}

impl MessageQueues {
    pub fn new() -> Self {
        Self {
            animations: DoubleBuffer::new(),
            changes: DoubleBuffer::new(),
            removed_entities: Vec::new(),
            take_cards: Vec::new(),
            player_moved_to: None,
        }
    }
}

pub trait PushMessages {
    fn animation(&mut self, animation: Animation);
    fn change(&mut self, change: EntityChange);
    fn remove(&mut self, entity_id: EntityId);
    fn take_card(&mut self, entity_id: EntityId, card: Card);
    fn move_player(&mut self, coord: Coord);
}

impl PushMessages for MessageQueues {
    fn animation(&mut self, animation: Animation) {
        self.animations.current.push(animation);
    }
    fn change(&mut self, change: EntityChange) {
        self.changes.current.push(change);
    }
    fn remove(&mut self, entity_id: EntityId) {
        self.removed_entities.push(entity_id);
    }
    fn take_card(&mut self, entity_id: EntityId, card: Card) {
        self.take_cards.push((entity_id, card));
    }
    fn move_player(&mut self, coord: Coord) {
        self.player_moved_to = Some(coord);
    }
}
