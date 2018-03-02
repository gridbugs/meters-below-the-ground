use std::time::Duration;
use std::collections::HashSet;
use grid_2d::{Coord, Size};
use entity_store::*;
use input::Input;
use policy;
use card::*;
use card_state::*;
use animation::*;
use rand::{Rng, SeedableRng, StdRng};
use pathfinding::PathfindingContext;
use message_queues::*;
use terrain::TerrainType;
use world::World;

const INITIAL_HAND_SIZE: usize = 4;

pub enum Meta {
    GameOver,
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
struct TickBuffers {
    ids_to_free: HashSet<EntityId>,
    npc_order: Vec<EntityId>,
    seen_animation_channels: HashSet<AnimationChannel>,
}

impl TickBuffers {
    fn new() -> Self {
        Self {
            ids_to_free: HashSet::new(),
            npc_order: Vec::new(),
            seen_animation_channels: HashSet::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct State {
    world: World,
    player_id: EntityId,
    card_state: CardState,
    input_state: InputState,
    rng: StdRng,
    turn: TurnState,
    messages: MessageQueues,
    swap_messages: MessageQueuesSwap,
    pathfinding: PathfindingContext,
    tick_buffers: TickBuffers,
    recompute_player_map: Option<Coord>,
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
    messages: MessageQueues,
}

impl State {
    pub fn new(rng_seed: usize) -> Self {
        let mut rng = StdRng::from_seed(&[rng_seed]);

        let terrain = TerrainType::FromStrings(
            vec![
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
        ]);

        let mut messages = MessageQueues::new();

        let world = World::new(&terrain, &mut messages);

        let player_id = *world.entity_store.player.iter().next()
            .expect("No player");

        let player_coord = Some(*world.entity_store.coord.get(&player_id)
            .expect("No player coord"));

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
            input_state: InputState::WaitingForCardSelection,
            player_id,
            card_state,
            rng,
            turn: TurnState::Player,
            recompute_player_map: player_coord,
            messages,
            swap_messages: MessageQueuesSwap::new(),
            pathfinding: PathfindingContext::new(world.size()),
            tick_buffers: TickBuffers::new(),
            world,
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
            world: World {
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
            messages,
            swap_messages: MessageQueuesSwap::new(),
            pathfinding: PathfindingContext::new(size),
            tick_buffers: TickBuffers::new(),
        }
    }

    pub fn create_save_state(&self, next_rng_seed: usize) -> SaveState {
        let mut changes = Vec::with_capacity(1024);
        self.world.entity_store.clone_changes(&mut changes);
        SaveState {
            changes,
            id_allocator: self.world.id_allocator.clone(),
            count: self.world.count,
            player_id: self.player_id,
            card_state: self.card_state.clone(),
            input_state: self.input_state.clone(),
            next_rng_seed,
            size: Size::new(
                self.world.spatial_hash.width(),
                self.world.spatial_hash.height(),
            ),
            turn: self.turn,
            recompute_player_map: self.recompute_player_map,
            messages: self.messages.clone(),
        }
    }

    pub fn entity_store(&self) -> &EntityStore {
        &self.world.entity_store
    }
    pub fn spatial_hash(&self) -> &SpatialHashTable {
        &self.world.spatial_hash
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
                                        &self.world.entity_store,
                                        direction,
                                        &mut self.world.id_allocator,
                                        &mut self.messages,
                                    );
                                    if policy::precheck(
                                        &self.messages.changes,
                                        &self.world.entity_store,
                                        &self.world.spatial_hash,
                                    ) {
                                        let card_to_check = self.card_state.hand.remove_card(index);
                                        assert_eq!(card, card_to_check);
                                        self.card_state.fill_hand();
                                        self.input_state = InputState::WaitingForCardSelection;
                                        self.turn = TurnState::Npcs;

                                        let ret = process_changes(
                                            &mut self.world,
                                            &mut self.card_state,
                                            &mut self.messages,
                                            &mut self.swap_messages,
                                            &mut self.recompute_player_map,
                                            &mut self.tick_buffers.ids_to_free,
                                            &mut self.rng,
                                        );

                                        self.world.count += 1;

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
                            .update_player_map(player_coord, &self.world.spatial_hash);
                    }

                    self.tick_buffers.npc_order.clear();
                    for &id in self.world.entity_store.npc.iter() {
                        self.tick_buffers.npc_order.push(id);
                    }

                    self.pathfinding.sort_entities_by_distance_to_player(
                        &self.world.entity_store,
                        &mut self.tick_buffers.npc_order,
                    );

                    for &id in self.tick_buffers.npc_order.iter() {
                        self.pathfinding.act(
                            id,
                            &self.world.entity_store,
                            &self.world.spatial_hash,
                            &mut self.messages,
                        );
                        if let Some(meta) = process_changes(
                            &mut self.world,
                            &mut self.card_state,
                            &mut self.messages,
                            &mut self.swap_messages,
                            &mut self.recompute_player_map,
                            &mut self.tick_buffers.ids_to_free,
                            &mut self.rng,
                        ) {
                            return Some(meta);
                        }
                    }

                    None
                }
            }
        } else {
            self.tick_buffers.seen_animation_channels.clear();

            for animation in swap_drain!(animations, self.messages, self.swap_messages) {
                let channel = animation.channel;
                if self.tick_buffers.seen_animation_channels.contains(&channel) {
                    self.messages.animations.push(animation);
                } else {
                    match animation.step(period, &mut self.messages) {
                        AnimationStatus::Continuing => {
                            self.tick_buffers.seen_animation_channels.insert(channel);
                        }
                        AnimationStatus::Finished => (),
                    }
                }
            }
            process_changes(
                &mut self.world,
                &mut self.card_state,
                &mut self.messages,
                &mut self.swap_messages,
                &mut self.recompute_player_map,
                &mut self.tick_buffers.ids_to_free,
                &mut self.rng,
            )
        }
    }
}

fn process_changes<R: Rng>(
    world: &mut World,
    card_state: &mut CardState,
    messages: &mut MessageQueues,
    swap_messages: &mut MessageQueuesSwap,
    recompute_player_map: &mut Option<Coord>,
    ids_to_free: &mut HashSet<EntityId>,
    rng: &mut R,
) -> Option<Meta> {

    let meta = loop {
        for id in messages.removed_entities.drain(..) {
            for component in world.entity_components.components(id) {
                messages.changes.push(EntityChange::Remove(id, component));
            }
            ids_to_free.insert(id);
        }

        if messages.changes.is_empty() {
            break None;
        }

        for change in swap_drain!(changes, messages, swap_messages) {

            if let &EntityChange::Insert(id, ..) = &change {
                if ids_to_free.contains(&id) {
                    continue;
                }
            }

            if !policy::check(
                &change,
                &world.entity_store,
                &world.spatial_hash,
                &mut world.id_allocator,
                messages,
            ) {
                continue;
            }

            world
                .spatial_hash
                .update(&world.entity_store, &change, world.count);
            world.entity_components.update(&change);
            world.entity_store.commit(change);
        }

        if let Some(coord) = messages.player_moved_to.take() {
            *recompute_player_map = Some(coord);
        }

        for (id, card) in messages.take_cards.drain(..) {
            card_state.deck.add_random(card, rng);
            messages.removed_entities.push(id);
        }
    };

    for id in ids_to_free.drain() {
        world.id_allocator.free(id);
    }

    meta
}
