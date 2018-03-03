use std::time::Duration;
use std::collections::{BTreeMap, HashSet};
use std::collections::btree_map;
use grid_2d::Size;
use entity_store::*;
use input::Input;
use policy;
use card::*;
use card_state::*;
use animation::*;
use rand::{SeedableRng, StdRng};
use pathfinding::PathfindingContext;
use message_queues::*;
use terrain::TerrainType;
use world::World;
use change::ChangeContext;
use event::*;
use meter::*;

const INITIAL_HAND_SIZE: usize = 4;

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

pub struct MeterInfoIter<'a> {
    entity_store: &'a EntityStore,
    entity_id: EntityId,
    meter_metadata: btree_map::Iter<'a, char, MeterType>,
}

impl<'a> Iterator for MeterInfoIter<'a> {
    type Item = MeterInfo;
    fn next(&mut self) -> Option<Self::Item> {
        self.meter_metadata.next().map(|(&identifier, &typ)| {
            let meter = Meter::from_entity_store(self.entity_id, self.entity_store, typ)
                .expect("Meter identifiers out of sync with game state");
            MeterInfo {
                typ,
                identifier,
                meter,
            }
        })
    }
}

#[derive(Clone, Debug)]
pub struct State {
    world: World,
    card_state: CardState,
    messages: MessageQueues,
    swap_messages: MessageQueuesSwap,
    npc_order: Vec<EntityId>,
    seen_animation_channels: HashSet<AnimationChannel>,
    rng: StdRng,
    player_id: EntityId,
    input_state: InputState,
    turn: TurnState,
    pathfinding: PathfindingContext,
    change_context: ChangeContext,
    meter_identifiers: BTreeMap<char, MeterType>,
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
    messages: MessageQueues,
    meter_identifiers: BTreeMap<char, MeterType>,
}

const METER_IDS: &'static str = "abcdefghijklmnopqrstuvwxyz";

impl State {
    pub fn switch_levels(&mut self) {
        let terrain = TerrainType::StaticStrings(vec![
            "##########",
            "#.....1..#",
            "#..m.....#",
            "#m..1....#",
            "#1.......#",
            "#..#####.#",
            "#........#",
            "#.....@>.#",
            "#........#",
            "##########",
        ]);

        let mut next_world = World::new(&terrain, &mut self.messages);

        let next_player_id = *next_world
            .entity_store
            .player
            .iter()
            .next()
            .expect("No player");

        for change in self.world
            .component_drain_insert(self.player_id, next_player_id)
        {
            if change.typ() == ComponentType::Coord {
                // otherwise the player would be moved to their old position in the new level
                continue;
            }

            next_world.commit(change);
        }

        let player_coord = *next_world
            .entity_store
            .coord
            .get(&next_player_id)
            .expect("No player coord");
        self.messages.player_moved_to = Some(player_coord);

        self.player_id = next_player_id;
        self.world = next_world;
        self.turn = TurnState::Player;
    }

    pub fn new(rng_seed: usize) -> Self {
        let mut rng = StdRng::from_seed(&[rng_seed]);

        let terrain = TerrainType::StaticStrings(vec![
            "##############################",
            "#............................#",
            "#.@..........................#",
            "#......1.....................#",
            "#............................#",
            "#............................#",
            "#............................#",
            "#............................#",
            "#............................#",
            "#............................#",
            "#............................#",
            "#............................#",
            "#............................#",
            "#............................#",
            "#............................#",
            "#............................#",
            "#............................#",
            "#............................#",
            "#............................#",
            "#............................#",
            "#............................#",
            "#............................#",
            "#............................#",
            "#............................#",
            "#............................#",
            "#............................#",
            "#............................#",
            "#............................#",
            "#............................#",
            "##############################",
        ]);

        let mut messages = MessageQueues::new();

        let world = World::new(&terrain, &mut messages);

        let player_id = *world.entity_store.player.iter().next().expect("No player");

        let player_coord = *world
            .entity_store
            .coord
            .get(&player_id)
            .expect("No player coord");
        messages.player_moved_to = Some(player_coord);

        let meter_identifiers: BTreeMap<_, _> = izip!(
            METER_IDS.chars(),
            world
                .entity_components
                .component_types(player_id)
                .filter_map(MeterType::from_component_type)
        ).collect();

        let card_state = CardState::new(
            vec![
                Card::Punch,
                Card::Punch,
                Card::Move,
                Card::Move,
                Card::Shoot,
                Card::Shoot,
                Card::Shoot,
                Card::Shoot,
                Card::Shoot,
                Card::Shoot,
                Card::Shoot,
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
            messages,
            swap_messages: MessageQueuesSwap::new(),
            pathfinding: PathfindingContext::new(world.size()),
            npc_order: Vec::new(),
            seen_animation_channels: HashSet::new(),
            change_context: ChangeContext::new(),
            world,
            meter_identifiers,
        }
    }

    pub fn save(&self, next_rng_seed: usize) -> SaveState {
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
            size: self.world.size(),
            turn: self.turn,
            messages: self.messages.clone(),
            meter_identifiers: self.meter_identifiers.clone(),
        }
    }

    pub fn player_meter_info(&self) -> MeterInfoIter {
        MeterInfoIter {
            entity_store: &self.world.entity_store,
            meter_metadata: self.meter_identifiers.iter(),
            entity_id: self.player_id,
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

    fn player_turn(&mut self, input: Input) -> Option<Event> {
        match input {
            Input::Direction(direction) => {
                let current = *self.world.entity_store.coord.get(&self.player_id).unwrap();
                let next = current + direction.coord();
                self.messages
                    .changes
                    .push(insert::coord(self.player_id, next));

                let ret = self.change_context.process(
                    &mut self.world,
                    &mut self.card_state,
                    &mut self.messages,
                    &mut self.swap_messages,
                    &mut self.rng,
                );

                self.world.count += 1;

                ret
            }
            Input::Wait => {
                self.turn = TurnState::Npcs;
                None
            }
        }
    }

    fn all_npc_turns(&mut self) -> Option<Event> {
        self.turn = TurnState::Player;

        if let Some(player_coord) = self.messages.player_moved_to.take() {
            self.pathfinding
                .update_player_map(player_coord, &self.world.spatial_hash);
        }

        self.npc_order.clear();
        for &id in self.world.entity_store.npc.iter() {
            self.npc_order.push(id);
        }

        self.pathfinding
            .sort_entities_by_distance_to_player(&self.world.entity_store, &mut self.npc_order);

        for &id in self.npc_order.iter() {
            self.pathfinding.act(
                id,
                &self.world.entity_store,
                &self.world.spatial_hash,
                &mut self.messages,
            );
            if let Some(meta) = self.change_context.process(
                &mut self.world,
                &mut self.card_state,
                &mut self.messages,
                &mut self.swap_messages,
                &mut self.rng,
            ) {
                return Some(meta);
            }
        }

        None
    }

    fn animation_tick(&mut self, period: Duration) -> Option<Event> {
        self.seen_animation_channels.clear();

        for animation in swap_drain!(animations, self.messages, self.swap_messages) {
            if let Some(channel) = animation.channel {
                if self.seen_animation_channels.contains(&channel) {
                    self.messages.animations.push(animation);
                    continue;
                }
            }

            match animation.step(period, &self.world.entity_store, &mut self.messages) {
                AnimationStatus::ContinuingOnChannel(channel) => {
                    self.seen_animation_channels.insert(channel);
                }
                AnimationStatus::Finished | AnimationStatus::Continuing => (),
            }
        }
        self.change_context.process(
            &mut self.world,
            &mut self.card_state,
            &mut self.messages,
            &mut self.swap_messages,
            &mut self.rng,
        )
    }

    pub fn tick<I>(&mut self, inputs: I, period: Duration) -> Option<ExternalEvent>
    where
        I: IntoIterator<Item = Input>,
    {
        let event = if self.messages.animations.is_empty() {
            match self.turn {
                TurnState::Player => {
                    if let Some(input) = inputs.into_iter().next() {
                        self.player_turn(input)
                    } else {
                        None
                    }
                }
                TurnState::Npcs => self.all_npc_turns(),
            }
        } else {
            self.animation_tick(period)
        };

        match event {
            Some(Event::External(external_event)) => Some(external_event),
            Some(Event::NextLevel) => {
                self.switch_levels();
                None
            }
            None => None,
        }
    }
}

impl From<SaveState> for State {
    fn from(
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
            messages,
            meter_identifiers,
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
            messages,
            swap_messages: MessageQueuesSwap::new(),
            pathfinding: PathfindingContext::new(size),
            npc_order: Vec::new(),
            seen_animation_channels: HashSet::new(),
            change_context: ChangeContext::new(),
            meter_identifiers,
        }
    }
}
