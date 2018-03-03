use std::time::Duration;
use std::collections::{BTreeMap, HashSet};
use std::collections::btree_map;
use grid_2d::Size;
use entity_store::*;
use input::Input;
use policy;
use animation::*;
use rand::{SeedableRng, StdRng};
use pathfinding::PathfindingContext;
use message_queues::*;
use terrain::*;
use world::World;
use change::ChangeContext;
use event::*;
use meter::*;
use goal::*;
use common_animations;
use prototypes;

#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
enum TurnState {
    Player,
    Npcs,
}

pub struct MeterInfoIter<'a> {
    entity_store: &'a EntityStore,
    entity_id: EntityId,
    meter_metadata: btree_map::Iter<'a, char, MeterType>,
    selected_meter: Option<SelectableMeterType>,
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
                is_selected: Some(typ) == self.selected_meter.map(|m| m.meter()),
            }
        })
    }
}

#[derive(Clone, Debug)]
pub struct State {
    world: World,
    messages: MessageQueues,
    swap_messages: MessageQueuesSwap,
    npc_order: Vec<EntityId>,
    seen_animation_channels: HashSet<AnimationChannel>,
    rng: StdRng,
    player_id: EntityId,
    turn: TurnState,
    pathfinding: PathfindingContext,
    change_context: ChangeContext,
    meter_identifiers: BTreeMap<char, MeterType>,
    selected_meter: Option<SelectableMeterType>,
    levels: Vec<TerrainInfo>,
    level_index: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaveState {
    changes: Vec<EntityChange>,
    id_allocator: EntityIdAllocator,
    count: u64,
    player_id: EntityId,
    next_rng_seed: usize,
    size: Size,
    turn: TurnState,
    messages: MessageQueues,
    meter_identifiers: BTreeMap<char, MeterType>,
    selected_meter: Option<SelectableMeterType>,
    levels: Vec<TerrainInfo>,
    level_index: usize,
}

const METER_IDS: &'static str = "abcdefghijklmnopqrstuvwxyz";

impl State {
    pub fn selected_meter_type(&self) -> Option<SelectableMeterType> {
        self.selected_meter
    }

    fn switch_levels(&mut self) {
        self.level_index += 1;
        let mut next_world = World::new(&self.levels[self.level_index], &mut self.messages);

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
        let rng = StdRng::from_seed(&[rng_seed]);

        let terrain = TerrainType::StaticStrings(vec![
            "##############################",
            "#............................#",
            "#.@.<........................#",
            "#......l.....................#",
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
        ].into_iter().map(|s| s.to_string()).collect());

        let first_terrain = TerrainInfo {
            typ: terrain,
            config: Default::default(),
        };

        let common_terrain = TerrainInfo {
            typ: TerrainType::Empty,
            config: Default::default(),
        };

        let final_terrain = TerrainInfo {
            typ: TerrainType::Empty,
            config: TerrainConfig {
                final_level: true,
            },
        };

        let levels = vec![
            first_terrain,
            common_terrain.clone(),
            common_terrain.clone(),
            final_terrain,
        ];

        let level_index = 0;

        let mut messages = MessageQueues::new();

        let world = World::new(&levels[level_index], &mut messages);

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

        Self {
            player_id,
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
            selected_meter: None,
            levels,
            level_index,
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
            next_rng_seed,
            size: self.world.size(),
            turn: self.turn,
            messages: self.messages.clone(),
            meter_identifiers: self.meter_identifiers.clone(),
            selected_meter: self.selected_meter,
            levels: self.levels.clone(),
            level_index: self.level_index,
        }
    }

    pub fn player_meter_info(&self) -> MeterInfoIter {
        MeterInfoIter {
            entity_store: &self.world.entity_store,
            meter_metadata: self.meter_identifiers.iter(),
            entity_id: self.player_id,
            selected_meter: self.selected_meter,
        }
    }

    pub fn entity_store(&self) -> &EntityStore {
        &self.world.entity_store
    }
    pub fn spatial_hash(&self) -> &SpatialHashTable {
        &self.world.spatial_hash
    }

    pub fn goal(&self) -> Goal {
        Goal::Escape
    }

    pub fn overall_progress_meter(&self) -> Meter {
        Meter {
            value: 12,
            max: 100,
        }
    }

    fn player_turn(&mut self, input: Input) -> Option<Event> {
        match input {
            Input::Direction(direction) => match self.selected_meter {
                None => {
                    let current = *self.world.entity_store.coord.get(&self.player_id).unwrap();
                    let next = current + direction.coord();
                    self.messages
                        .changes
                        .push(insert::coord(self.player_id, next));
                }
                Some(SelectableMeterType::GunAmmo) => {
                    let mut ammo = self.world
                        .entity_store
                        .gun_ammo_meter
                        .get(&self.player_id)
                        .cloned()
                        .unwrap();

                    if ammo.value > 0 {
                        let entity_coord = self.world
                            .entity_store
                            .coord
                            .get(&self.player_id)
                            .cloned()
                            .unwrap();
                        let start_coord = entity_coord + direction.coord();
                        let bullet_id = self.world.id_allocator.allocate();
                        prototypes::bullet(bullet_id, start_coord, direction, &mut self.messages);
                        common_animations::bullet(bullet_id, &mut self.messages);
                        ammo.value -= 1;
                        self.messages
                            .change(insert::gun_ammo_meter(self.player_id, ammo));
                    }
                }
            },
            Input::MeterSelect(identifier) => {
                if let Some(meter_type) = self.meter_identifiers.get(&identifier).cloned() {
                    self.selected_meter = meter_type.selectable();
                }
                return None;
            }
            Input::MeterDeselect => {
                self.selected_meter = None;
                return None;
            }
            Input::Wait => (),
        }

        if !policy::precheck(
            &self.messages.changes,
            &self.world.entity_store,
            &self.world.spatial_hash,
        ) {
            self.messages.changes.clear();
            return None;
        }

        self.turn = TurnState::Npcs;

        let ret = self.change_context.process(
            &mut self.world,
            &mut self.messages,
            &mut self.swap_messages,
        );

        self.world.count += 1;

        ret
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
                &mut self.messages,
                &mut self.swap_messages,
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
        self.change_context
            .process(&mut self.world, &mut self.messages, &mut self.swap_messages)
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
            next_rng_seed,
            size,
            turn,
            messages,
            meter_identifiers,
            selected_meter,
            levels,
            level_index,
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
            player_id,
            rng: StdRng::from_seed(&[next_rng_seed]),
            turn,
            messages,
            swap_messages: MessageQueuesSwap::new(),
            pathfinding: PathfindingContext::new(size),
            npc_order: Vec::new(),
            seen_animation_channels: HashSet::new(),
            change_context: ChangeContext::new(),
            meter_identifiers,
            selected_meter,
            levels,
            level_index,
        }
    }
}
