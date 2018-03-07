use entity_store::*;
use animation::*;

macro_rules! swap_drain {
    ($field:ident, $current:expr, $swap:expr) => {
        {
            ::std::mem::swap(&mut $current.$field, &mut $swap.$field);
            $swap.$field.drain(..)
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Special {
    Lose,
    Win,
    Ascend,
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
    pub player_moved_to: Option<Coord>,
    pub special: Option<Special>,
}

impl MessageQueues {
    pub fn new() -> Self {
        Self {
            animations: Vec::new(),
            changes: Vec::new(),
            removed_entities: Vec::new(),
            player_moved_to: None,
            special: None,
        }
    }
    pub fn clear(&mut self) {
        self.animations.clear();
        self.changes.clear();
        self.removed_entities.clear();
        self.player_moved_to = None;
        self.special = None;
    }
}

pub trait PushMessages {
    fn animate(&mut self, animation: Animation);
    fn change(&mut self, change: EntityChange);
    fn remove(&mut self, entity_id: EntityId);
    fn move_player(&mut self, coord: Coord);
    fn lose(&mut self);
    fn win(&mut self);
    fn ascend(&mut self);
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
    fn move_player(&mut self, coord: Coord) {
        self.player_moved_to = Some(coord);
    }
    fn lose(&mut self) {
        self.special = Some(Special::Lose);
    }
    fn win(&mut self) {
        self.special = Some(Special::Win);
    }
    fn ascend(&mut self) {
        self.special = Some(Special::Ascend);
    }
}
