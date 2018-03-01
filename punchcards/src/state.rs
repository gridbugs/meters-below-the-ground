use std::time::Duration;
use std::collections::HashSet;
use grid_2d::{Coord, Size};
use entity_store::*;
use input::Input;
use policy;
use prototypes;
use card::*;
use card_state::*;
use tile::*;
use animation::*;
use rand::{Rng, SeedableRng, StdRng};
use pathfinding::PathfindingContext;
use message_queues::*;

const INITIAL_HAND_SIZE: usize = 4;

pub enum Meta {
    GameOver,
}

#[derive(Clone, Debug)]
struct World {
    entity_store: EntityStore,
    spatial_hash: SpatialHashTable,
    entity_components: EntityComponentTable,
    id_allocator: EntityIdAllocator,
    count: u64,
}

impl World {
    fn delete_entity(&self, entity_id: EntityId, changes: &mut Vec<EntityChange>) {
        for component in self.entity_components.components(entity_id) {
            changes.push(EntityChange::Remove(entity_id, component));
        }
    }
    fn add_changes_for_removed_entities(&self, messages: &mut MessageQueues) {
        for id in messages.removed_entities.drain(..) {
            self.delete_entity(id, &mut messages.changes);
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum InputState {
    WaitingForCardSelection,
    WaitingForDirection(HandIndex, Card),
}

#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
enum TurnState {
    Player,
    Npcs,
}

#[derive(Clone, Debug)]
pub struct State {
    game_state: World,
    player_id: EntityId,
    card_state: CardState,
    input_state: InputState,
    rng: StdRng,
    turn: TurnState,
    recompute_player_map: Option<Coord>,
    npc_order: Vec<EntityId>,
    seen_animation_channels: HashSet<AnimationChannel>,
    messages: MessageQueues,
    swap_messages: MessageQueuesSwap,
    pathfinding: PathfindingContext,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaveState {
    changes: Vec<EntityChange>,
    id_allocator: EntityIdAllocator,
    count: u64,
    player_id: EntityId,
    card_state: CardState,
    input_state: InputState,
    next_rng_seed: usize,
    size: Size,
    turn: TurnState,
    recompute_player_map: Option<Coord>,
    seen_animation_channels: HashSet<AnimationChannel>,
    messages: MessageQueues,
}

impl State {
    pub fn new(rng_seed: usize) -> Self {
        let mut rng = StdRng::from_seed(&[rng_seed]);

        let strings = vec![
            "##########",
            "#@...1111#",
            "#m....#11#",
            "#.....#11#",
            "#.....#..#",
            "###.###..#",
            "#.1111...#",
            "#........#",
            "#........#",
            "##########",
        ];

        let size = Size::new(strings[0].len() as u32, strings.len() as u32);

        let mut entity_store = EntityStore::new();
        let mut spatial_hash = SpatialHashTable::new(size);
        let mut id_allocator = EntityIdAllocator::new();
        let mut player_id = None;
        let mut player_coord = None;
        let mut messages = MessageQueues::new();

        for (y, line) in strings.iter().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                let coord = Coord::new(x as i32, y as i32);
                match ch {
                    '#' => {
                        prototypes::wall(id_allocator.allocate(), coord, &mut messages);
                        prototypes::floor(id_allocator.allocate(), coord, &mut messages);
                    }
                    '.' => {
                        prototypes::floor(id_allocator.allocate(), coord, &mut messages);
                    }
                    'm' => {
                        prototypes::card(
                            id_allocator.allocate(),
                            coord,
                            Card::Move,
                            Tile::CardMove,
                            &mut messages,
                        );
                        prototypes::floor(id_allocator.allocate(), coord, &mut messages);
                    }
                    '0' => {
                        prototypes::target_dummy(id_allocator.allocate(), coord, &mut messages);
                        prototypes::floor(id_allocator.allocate(), coord, &mut messages);
                    }
                    '1' => {
                        prototypes::small_robot(id_allocator.allocate(), coord, &mut messages);
                        prototypes::floor(id_allocator.allocate(), coord, &mut messages);
                    }
                    '@' => {
                        let id = id_allocator.allocate();
                        player_id = Some(id);
                        player_coord = Some(coord);
                        prototypes::player(id, coord, &mut messages);
                        prototypes::floor(id_allocator.allocate(), coord, &mut messages);
                    }
                    _ => panic!("unexpected character"),
                }
            }
        }

        let player_id = player_id.expect("No player in level");

        let mut entity_components = EntityComponentTable::new();

        for change in messages.changes.drain(..) {
            spatial_hash.update(&entity_store, &change, 0);
            entity_components.update(&change);
            entity_store.commit(change);
        }

        let card_state = CardState::new(
            vec![
                Card::Punch,
                Card::Punch,
                Card::Punch,
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
                Card::Move,
                Card::Move,
            ],
            INITIAL_HAND_SIZE,
            &mut rng,
        );

        Self {
            game_state: World {
                entity_store,
                spatial_hash,
                entity_components,
                id_allocator,
                count: 0,
            },
            input_state: InputState::WaitingForCardSelection,
            player_id,
            card_state,
            rng,
            turn: TurnState::Player,
            recompute_player_map: player_coord,
            npc_order: Vec::new(),
            seen_animation_channels: HashSet::new(),
            messages,
            swap_messages: MessageQueuesSwap::new(),
            pathfinding: PathfindingContext::new(size),
        }
    }

    pub fn from_save_state(
        SaveState {
            mut changes,
            id_allocator,
            count,
            player_id,
            card_state,
            input_state,
            next_rng_seed,
            size,
            turn,
            recompute_player_map,
            seen_animation_channels,
            messages,
        }: SaveState,
    ) -> Self {
        let mut entity_store = EntityStore::new();
        let mut spatial_hash = SpatialHashTable::new(size);
        let mut entity_components = EntityComponentTable::new();

        for change in changes.drain(..) {
            spatial_hash.update(&entity_store, &change, 0);
            entity_components.update(&change);
            entity_store.commit(change);
        }

        Self {
            game_state: World {
                entity_store,
                spatial_hash,
                entity_components,
                id_allocator,
                count,
            },
            input_state,
            player_id,
            card_state,
            rng: StdRng::from_seed(&[next_rng_seed]),
            turn,
            recompute_player_map,
            npc_order: Vec::new(),
            seen_animation_channels,
            messages,
            swap_messages: MessageQueuesSwap::new(),
            pathfinding: PathfindingContext::new(size),
        }
    }

    pub fn create_save_state(&self, next_rng_seed: usize) -> SaveState {
        let mut changes = Vec::with_capacity(1024);
        self.game_state.entity_store.clone_changes(&mut changes);
        SaveState {
            changes,
            id_allocator: self.game_state.id_allocator.clone(),
            count: self.game_state.count,
            player_id: self.player_id,
            card_state: self.card_state.clone(),
            input_state: self.input_state.clone(),
            next_rng_seed,
            size: Size::new(
                self.game_state.spatial_hash.width(),
                self.game_state.spatial_hash.height(),
            ),
            turn: self.turn,
            recompute_player_map: self.recompute_player_map,
            seen_animation_channels: self.seen_animation_channels.clone(),
            messages: self.messages.clone(),
        }
    }

    pub fn entity_store(&self) -> &EntityStore {
        &self.game_state.entity_store
    }
    pub fn spatial_hash(&self) -> &SpatialHashTable {
        &self.game_state.spatial_hash
    }
    pub fn card_state(&self) -> &CardState {
        &self.card_state
    }
    pub fn input_state(&self) -> &InputState {
        &self.input_state
    }

    pub fn tick<I>(&mut self, inputs: I, period: Duration) -> Option<Meta>
    where
        I: IntoIterator<Item = Input>,
    {
        if self.messages.animations.is_empty() {
            match self.turn {
                TurnState::Player => {
                    if let Some(input) = inputs.into_iter().next() {
                        match input {
                            Input::SelectCard(index) => {
                                if let Some(card) = self.card_state.hand.get(index) {
                                    self.input_state =
                                        InputState::WaitingForDirection(index, *card);
                                }
                                None
                            }
                            Input::Direction(direction) => {
                                if let InputState::WaitingForDirection(index, card) =
                                    self.input_state
                                {
                                    card.play(
                                        self.player_id,
                                        &self.game_state.entity_store,
                                        direction,
                                        &mut self.game_state.id_allocator,
                                        &mut self.messages,
                                    );
                                    if policy::precheck(
                                        &self.messages.changes,
                                        &self.game_state.entity_store,
                                        &self.game_state.spatial_hash,
                                    ) {
                                        let card_to_check = self.card_state.hand.remove_card(index);
                                        assert_eq!(card, card_to_check);
                                        self.card_state.fill_hand();
                                        self.input_state = InputState::WaitingForCardSelection;
                                        self.turn = TurnState::Npcs;

                                        let ret = process_changes(
                                            &mut self.game_state,
                                            &mut self.card_state,
                                            &mut self.messages,
                                            &mut self.swap_messages,
                                            &mut self.recompute_player_map,
                                            &mut self.rng,
                                        );

                                        self.game_state.count += 1;

                                        ret
                                    } else {
                                        self.messages.changes.clear();
                                        None
                                    }
                                } else {
                                    None
                                }
                            }
                            Input::Wait => {
                                self.turn = TurnState::Npcs;
                                None
                            }
                        }
                    } else {
                        None
                    }
                }
                TurnState::Npcs => {
                    self.turn = TurnState::Player;

                    if let Some(player_coord) = self.recompute_player_map.take() {
                        self.pathfinding
                            .update_player_map(player_coord, &self.game_state.spatial_hash);
                    }

                    self.npc_order.clear();
                    for &id in self.game_state.entity_store.npc.iter() {
                        self.npc_order.push(id);
                    }

                    self.pathfinding.sort_entities_by_distance_to_player(
                        &self.game_state.entity_store,
                        &mut self.npc_order,
                    );

                    for &id in self.npc_order.iter() {
                        self.pathfinding.act(
                            id,
                            &self.game_state.entity_store,
                            &self.game_state.spatial_hash,
                            &mut self.messages,
                        );
                        if let Some(meta) = process_changes(
                            &mut self.game_state,
                            &mut self.card_state,
                            &mut self.messages,
                            &mut self.swap_messages,
                            &mut self.recompute_player_map,
                            &mut self.rng,
                        ) {
                            return Some(meta);
                        }
                    }

                    None
                }
            }
        } else {
            self.seen_animation_channels.clear();

            for animation in swap_drain!(animations, self.messages, self.swap_messages) {
                let channel = animation.channel;
                if self.seen_animation_channels.contains(&channel) {
                    self.messages.animations.push(animation);
                } else {
                    match animation.step(period, &mut self.messages) {
                        AnimationStatus::Continuing => {
                            self.seen_animation_channels.insert(channel);
                        }
                        AnimationStatus::Finished => (),
                    }
                }
            }
            process_changes(
                &mut self.game_state,
                &mut self.card_state,
                &mut self.messages,
                &mut self.swap_messages,
                &mut self.recompute_player_map,
                &mut self.rng,
            )
        }
    }
}

fn process_changes<R: Rng>(
    game_state: &mut World,
    card_state: &mut CardState,
    messages: &mut MessageQueues,
    swap_messages: &mut MessageQueuesSwap,
    recompute_player_map: &mut Option<Coord>,
    rng: &mut R,
) -> Option<Meta> {
    loop {
        game_state.add_changes_for_removed_entities(messages);

        if messages.changes.is_empty() {
            break None;
        }

        for change in swap_drain!(changes, messages, swap_messages) {
            if !policy::check(
                &change,
                &game_state.entity_store,
                &game_state.spatial_hash,
                &mut game_state.id_allocator,
                messages,
            ) {
                continue;
            }

            game_state
                .spatial_hash
                .update(&game_state.entity_store, &change, game_state.count);
            game_state.entity_components.update(&change);
            game_state.entity_store.commit(change);
        }

        if let Some(coord) = messages.player_moved_to.take() {
            *recompute_player_map = Some(coord);
        }

        for (id, card) in messages.take_cards.drain(..) {
            card_state.deck.add_random(card, rng);
            game_state.delete_entity(id, &mut messages.changes);
        }
    }
}
