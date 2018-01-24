use std::time::Duration;
use entity_store::*;
use input::Input;
use policy;
use cgmath::*;
use prototypes;
use card::*;
use card_state::*;
use tile::*;
use reaction::*;
use animation::*;
use rand::Rng;
use append::Append;

pub enum Meta {
    GameOver,
}

pub struct GameState {
    entity_store: EntityStore,
    spatial_hash: SpatialHashTable,
    entity_components: EntityComponentTable,
    id_allocator: EntityIdAllocator,
    count: u64,
}

impl GameState {
    fn delete_entity<A: Append<EntityChange>>(&mut self, entity_id: EntityId, changes: &mut A) {
        for component in self.entity_components.components(entity_id) {
            changes.append(EntityChange::Remove(entity_id, component));
        }
    }
}


pub struct State<R: Rng> {
    game_state: GameState,
    player_id: EntityId,
    changes: Vec<EntityChange>,
    reactions: Vec<Reaction>,
    animations: Vec<Animation>,
    card_state: CardState,
    rng: R,
}


impl<R: Rng> State<R> {
    pub fn new(mut rng: R) -> Self {

        let strings = vec![
            "##########",
            "#..m.....#",
            "#....#...#",
            "#..@.#...#",
            "#....#...#",
            "#.####...#",
            "#........#",
            "#........#",
            "#........#",
            "##########",
        ];

        let mut entity_store = EntityStore::new();
        let mut spatial_hash = SpatialHashTable::new(strings[0].len() as u32, strings.len() as u32);
        let mut id_allocator = EntityIdAllocator::new();
        let mut changes = Vec::new();
        let animations = Vec::new();
        let mut player_id = None;

        for (y, line) in strings.iter().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                let coord = vec2(x as i32, y as i32);
                match ch {
                    '#' => {
                        prototypes::wall(id_allocator.allocate(), coord, &mut changes);
                        prototypes::floor(id_allocator.allocate(), coord, &mut changes);
                    }
                    '.' => {
                        prototypes::floor(id_allocator.allocate(), coord, &mut changes);
                    }
                    'm' => {
                        prototypes::card(id_allocator.allocate(), coord, Card::Move, Tile::CardMove, &mut changes);
                        prototypes::floor(id_allocator.allocate(), coord, &mut changes);
                    }
                    '@' => {
                        let id = id_allocator.allocate();
                        player_id = Some(id);
                        prototypes::player(id, coord, &mut changes);
                        prototypes::floor(id_allocator.allocate(), coord, &mut changes);
                    }
                    _ => panic!("unexpected character"),
                }
            }
        }

        let player_id = player_id.expect("No player in level");

        let mut entity_components = EntityComponentTable::new();

        for change in changes.drain(..) {
            spatial_hash.update(&entity_store, &change, 0);
            entity_components.update(&change);
            entity_store.commit(change);
        }

        let card_state = CardState::new(vec![
            Card::Punch,
            Card::Punch,
            Card::Punch,
            Card::Punch,
            Card::Punch,
            Card::Move,
            Card::Move,
            Card::Move,
            Card::Move,
            Card::Move,
        ], &mut rng);

        Self {
            game_state: GameState {
                entity_store,
                spatial_hash,
                entity_components,
                id_allocator,
                count: 0,
            },
            player_id,
            changes,
            animations,
            reactions: Vec::new(),
            card_state,
            rng,
        }
    }

    pub fn entity_store(&self) -> &EntityStore { &self.game_state.entity_store }
    pub fn spatial_hash(&self) -> &SpatialHashTable { &self.game_state.spatial_hash }
    pub fn card_state(&self) -> &CardState { &self.card_state }

    pub fn tick<I>(&mut self, inputs: I, period: Duration) -> Option<Meta>
        where I: IntoIterator<Item=Input>,
    {

        if self.animations.is_empty() {
            for input in inputs {
                match input {
                    Input::Move(direction) => {
                        let card = self.card_state.play(direction);
                        card.play(self.player_id, &self.game_state.entity_store,
                                  direction, &mut self.game_state.id_allocator,
                                  &mut self.changes, &mut self.reactions);
                    }
                };
            }
        } else {
            for animation in self.animations.drain(..) {
                animation.step(period, &mut self.reactions);
            }
        }

        loop {
            for change in self.changes.drain(..) {

                if !policy::check(&change, &self.game_state.entity_store, &self.game_state.spatial_hash, &mut self.reactions) {
                    continue;
                }
                self.game_state.spatial_hash.update(&self.game_state.entity_store, &change, self.game_state.count);
                self.game_state.entity_components.update(&change);
                self.game_state.entity_store.commit(change);
                self.game_state.count += 1;
            }

            if self.reactions.is_empty() {
                if self.card_state.hand_is_full() {
                    break None;
                } else {
                    break Some(Meta::GameOver);
                }
            } else {
                for reaction in self.reactions.drain(..) {
                    match reaction {
                        Reaction::TakeCard(entity_id, card) => {
                            self.card_state.add_card(card, &mut self.rng);
                            self.game_state.delete_entity(entity_id, &mut self.changes);
                        }
                        Reaction::RemoveEntity(entity_id) => {
                            self.game_state.delete_entity(entity_id, &mut self.changes);
                        }
                        Reaction::StartAnimation(animation) => {
                            self.animations.push(animation);
                        }
                        Reaction::EntityChange(change) => {
                            self.changes.push(change);
                        }
                    }
                }
            }
        }
    }
}
