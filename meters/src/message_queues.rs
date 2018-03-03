use entity_store::*;
use animation::*;
use card::*;

macro_rules! swap_drain {
    ($field:ident, $current:expr, $swap:expr) => {
        {
            ::std::mem::swap(&mut $current.$field, &mut $swap.$field);
            $swap.$field.drain(..)
        }
    }
}

#[derive(Clone, Debug)]
pub struct MessageQueuesSwap {
    pub animations: Vec<Animation>,
    pub changes: Vec<EntityChange>,
}

impl MessageQueuesSwap {
    pub fn new() -> Self {
        Self {
            animations: Vec::new(),
            changes: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MessageQueues {
    pub animations: Vec<Animation>,
    pub changes: Vec<EntityChange>,
    pub removed_entities: Vec<EntityId>,
    pub take_cards: Vec<(EntityId, Card)>,
    pub player_moved_to: Option<Coord>,
    pub next_level: bool,
    pub game_over: bool,
}

impl MessageQueues {
    pub fn new() -> Self {
        Self {
            animations: Vec::new(),
            changes: Vec::new(),
            removed_entities: Vec::new(),
            take_cards: Vec::new(),
            player_moved_to: None,
            next_level: false,
            game_over: false,
        }
    }
}

pub trait PushMessages {
    fn animate(&mut self, animation: Animation);
    fn change(&mut self, change: EntityChange);
    fn remove(&mut self, entity_id: EntityId);
    fn take_card(&mut self, entity_id: EntityId, card: Card);
    fn move_player(&mut self, coord: Coord);
    fn next_level(&mut self);
    fn game_over(&mut self);
}

impl PushMessages for MessageQueues {
    fn animate(&mut self, animation: Animation) {
        self.animations.push(animation);
    }
    fn change(&mut self, change: EntityChange) {
        self.changes.push(change);
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
    fn next_level(&mut self) {
        self.next_level = true;
    }
    fn game_over(&mut self) {
        self.game_over = true;
    }
}
