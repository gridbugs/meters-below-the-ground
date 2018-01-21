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

pub struct State {
    entity_store: EntityStore,
    spatial_hash: SpatialHashTable,
    entity_components: EntityComponentTable,
    player_id: EntityId,
    changes: Vec<EntityChange>,
    reactions: Vec<Reaction>,
    count: u64,
    card_state: CardState,
}


impl State {
    pub fn new() -> Self {

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
            Card::Move,
            Card::Move,
            Card::Move,
            Card::Move,
            Card::Move,
            Card::Move,
            Card::Move,
            Card::Move,
            Card::Move,
            Card::Move,
        ]);

        Self {
            entity_store,
            spatial_hash,
            entity_components,
            player_id,
            changes,
            reactions: Vec::new(),
            count: 0,
            card_state,
        }
    }

    pub fn entity_store(&self) -> &EntityStore { &self.entity_store }
    pub fn spatial_hash(&self) -> &SpatialHashTable { &self.spatial_hash }
    pub fn card_state(&self) -> &CardState { &self.card_state }

    pub fn tick<I>(&mut self, inputs: I, _period: Duration)
        where I: IntoIterator<Item=Input>,
    {
        for input in inputs {
            match input {
                Input::Move(direction) => {
                    if let Some(card) = self.card_state.play(direction) {
                        card.play(self.player_id, &self.entity_store, direction, &mut self.changes);
                    }
                }
            };
        }

        loop {
            for change in self.changes.drain(..) {

                if !policy::check(&change, &self.entity_store, &self.spatial_hash, &mut self.reactions) {
                    continue;
                }
                self.spatial_hash.update(&self.entity_store, &change, self.count);
                self.entity_components.update(&change);
                self.entity_store.commit(change);
                self.count += 1;
            }

            if self.reactions.is_empty() {
                break;
            } else {
                for reaction in self.reactions.drain(..) {
                    match reaction {
                        Reaction::TakeCard(entity_id, card) => {
                            self.card_state.add_top(card);
                            for component in self.entity_components.components(entity_id) {
                                self.changes.push(EntityChange::Remove(entity_id, component));
                            }
                        }
                    }
                }
            }
        }
    }
}
