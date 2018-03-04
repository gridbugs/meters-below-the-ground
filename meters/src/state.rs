use std::time::Duration;
use std::collections::HashSet;
use std::slice;
use std::iter::Enumerate;
use grid_2d::Size;
use entity_store::*;
use input::*;
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
use shadowcast::{self, ShadowcastContext};
use tile_info::*;
use grid_2d::*;
use grid_2d;
use direction::DirectionBitmap;

#[derive(Default, Debug, Clone)]
struct VisibilityCell {
    tiles: Vec<TileInfo>,
    last_updated: u64,
}

#[derive(Debug, Clone)]
struct VisibilityGrid(Grid<VisibilityCell>);

pub struct VisibilityIter<'a> {
    iter: grid_2d::CoordEnumerate<'a, VisibilityCell>,
    time: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Visible,
    Remembered,
}

impl<'a> Iterator for VisibilityIter<'a> {
    type Item = (slice::Iter<'a, TileInfo>, Coord, Visibility);
    fn next(&mut self) -> Option<Self::Item> {
        while let Some((coord, cell)) = self.iter.next() {
            let visibility = if cell.last_updated == 0 {
                continue;
            } else if cell.last_updated == self.time {
                Visibility::Visible
            } else {
                Visibility::Remembered
            };
            return Some((cell.tiles.iter(), coord, visibility));
        }
        None
    }
}

impl VisibilityGrid {
    fn new(size: Size) -> Self {
        VisibilityGrid(Grid::new_default(size))
    }
    fn iter(&self, time: u64) -> VisibilityIter {
        VisibilityIter {
            iter: self.0.enumerate(),
            time,
        }
    }
    fn clear(&mut self) {
        for cell in self.0.iter_mut() {
            cell.last_updated = 0;
            cell.tiles.clear();
        }
    }
}

struct VisibilityRefs<'a> {
    grid: &'a mut VisibilityGrid,
    world: &'a World,
}

impl<'a> shadowcast::OutputGrid for VisibilityRefs<'a> {
    fn see(&mut self, coord: Coord, _: DirectionBitmap, time: u64) {
        if let Some(cell) = self.grid.0.get_mut(coord) {
            if let Some(sh_cell) = self.world.spatial_hash.get(coord) {
                if sh_cell.last_updated > cell.last_updated {
                    cell.tiles.clear();
                    for id in sh_cell.tile_set.iter() {
                        if let Some(&tile_info) = self.world.entity_store.tile_info.get(&id) {
                            cell.tiles.push(tile_info);
                        }
                    }
                }
                cell.last_updated = time;
            }
        }
    }
}

impl shadowcast::InputGrid for SpatialHashTable {
    type Opacity = u8;
    type Visibility = u8;
    fn size(&self) -> Size {
        SpatialHashTable::size(self)
    }
    fn get_opacity(&self, coord: Coord) -> Option<Self::Opacity> {
        self.get(coord).map(|cell| cell.opacity_total)
    }
    fn initial_visibility() -> Self::Visibility {
        1
    }
}

#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
enum TurnState {
    Player,
    Npcs,
}

#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
enum PlayerTurnEvent {
    ChangeActiveMeter(ActiveMeterType, i32),
    ChangePassiveMeter(PassiveMeterType, i32),
}

#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
struct PlayerTurnEventEntry {
    event: PlayerTurnEvent,
    remaining: u32,
    reset: u32,
}

impl PlayerTurnEventEntry {
    fn full(event: PlayerTurnEvent, reset: u32) -> Self {
        Self {
            event,
            remaining: reset,
            reset,
        }
    }
}

pub struct ActiveMeterInfoIter<'a> {
    entity_store: &'a EntityStore,
    entity_id: EntityId,
    meter_metadata: Enumerate<slice::Iter<'a, ActiveMeterType>>,
    selected_meter: Option<ActiveMeterType>,
}

impl<'a> Iterator for ActiveMeterInfoIter<'a> {
    type Item = ActiveMeterInfo;
    fn next(&mut self) -> Option<Self::Item> {
        self.meter_metadata.next().map(|(index, &typ)| {
            let meter = Meter::from_entity_store(self.entity_id, self.entity_store, typ)
                .expect("Meter identifiers out of sync with game state");
            ActiveMeterInfo {
                typ,
                identifier: ActiveMeterIdentifier::from_index(index),
                meter,
                is_selected: Some(typ) == self.selected_meter,
            }
        })
    }
}

pub struct PassiveMeterInfoIter<'a> {
    entity_store: &'a EntityStore,
    entity_id: EntityId,
    meter_metadata: slice::Iter<'a, PassiveMeterType>,
}

impl<'a> Iterator for PassiveMeterInfoIter<'a> {
    type Item = PassiveMeterInfo;
    fn next(&mut self) -> Option<Self::Item> {
        self.meter_metadata.next().map(|&typ| {
            let meter = Meter::from_entity_store(self.entity_id, self.entity_store, typ)
                .expect("Meter list out of sync with game state");
            PassiveMeterInfo { typ, meter }
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
    active_meters: Vec<ActiveMeterType>,
    passive_meters: Vec<PassiveMeterType>,
    selected_meter: Option<ActiveMeterType>,
    levels: Vec<TerrainInfo>,
    level_index: usize,
    player_turn_events: Vec<PlayerTurnEventEntry>,
    shadowcast: ShadowcastContext<u8>,
    visibility_grid: VisibilityGrid,
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
    active_meters: Vec<ActiveMeterType>,
    passive_meters: Vec<PassiveMeterType>,
    levels: Vec<TerrainInfo>,
    level_index: usize,
    player_turn_events: Vec<PlayerTurnEventEntry>,
}

impl State {
    pub fn selected_meter_type(&self) -> Option<ActiveMeterType> {
        self.selected_meter
    }

    fn switch_levels(&mut self) {
        self.level_index += 1;
        let mut next_world = World::new(
            &self.levels[self.level_index],
            &mut self.messages,
            &mut self.rng,
        );

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

        if self.level_index == 1 {
            next_world.commit(insert::kevlar_meter(next_player_id, Meter::full(6)));
            next_world.commit(insert::medkit_meter(next_player_id, Meter::empty(5)));
            self.passive_meters.push(PassiveMeterType::Kevlar);
            self.active_meters.push(ActiveMeterType::Medkit);
            if let Some(change) = PassiveMeterType::Kevlar.periodic_change() {
                let event =
                    PlayerTurnEvent::ChangePassiveMeter(PassiveMeterType::Kevlar, change.change);
                let entry = PlayerTurnEventEntry::full(event, change.turns);
                self.player_turn_events.push(entry);
            }
            if let Some(change) = ActiveMeterType::Medkit.periodic_change() {
                let event =
                    PlayerTurnEvent::ChangeActiveMeter(ActiveMeterType::Medkit, change.change);
                let entry = PlayerTurnEventEntry::full(event, change.turns);
                self.player_turn_events.push(entry);
            }
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

        self.visibility_grid.clear();
        self.update_visibility();
    }

    pub fn new(rng_seed: usize) -> Self {
        let mut rng = StdRng::from_seed(&[rng_seed]);

        let common_terrain = TerrainInfo {
            typ: TerrainType::Dungeon,
            config: Default::default(),
        };

        let final_terrain = TerrainInfo {
            typ: TerrainType::Dungeon,
            config: TerrainConfig { final_level: true },
        };

        let levels = vec![
            common_terrain.clone(),
            common_terrain.clone(),
            common_terrain.clone(),
            final_terrain,
        ];

        let level_index = 0;

        let mut messages = MessageQueues::new();

        let world = World::new(&levels[level_index], &mut messages, &mut rng);

        let player_id = *world.entity_store.player.iter().next().expect("No player");

        let player_coord = *world
            .entity_store
            .coord
            .get(&player_id)
            .expect("No player coord");
        messages.player_moved_to = Some(player_coord);

        let active_meters: Vec<_> = world
            .entity_components
            .component_types(player_id)
            .filter_map(ActiveMeterType::from_component_type)
            .collect();

        let passive_meters: Vec<_> = world
            .entity_components
            .component_types(player_id)
            .filter_map(PassiveMeterType::from_component_type)
            .collect();

        let mut player_turn_events = Vec::new();

        for typ in active_meters.iter().cloned() {
            if let Some(change) = typ.periodic_change() {
                let event = PlayerTurnEvent::ChangeActiveMeter(typ, change.change);
                let entry = PlayerTurnEventEntry::full(event, change.turns);
                player_turn_events.push(entry);
            }
        }

        for typ in passive_meters.iter().cloned() {
            if let Some(change) = typ.periodic_change() {
                let event = PlayerTurnEvent::ChangePassiveMeter(typ, change.change);
                let entry = PlayerTurnEventEntry::full(event, change.turns);
                player_turn_events.push(entry);
            }
        }

        Self {
            player_id,
            rng,
            turn: TurnState::Player,
            messages,
            swap_messages: MessageQueuesSwap::new(),
            pathfinding: PathfindingContext::new(world.size()),
            visibility_grid: VisibilityGrid::new(world.size()),
            npc_order: Vec::new(),
            seen_animation_channels: HashSet::new(),
            change_context: ChangeContext::new(),
            world,
            active_meters,
            passive_meters,
            selected_meter: None,
            levels,
            level_index,
            player_turn_events,
            shadowcast: ShadowcastContext::new(),
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
            active_meters: self.active_meters.clone(),
            passive_meters: self.passive_meters.clone(),
            levels: self.levels.clone(),
            level_index: self.level_index,
            player_turn_events: self.player_turn_events.clone(),
        }
    }

    pub fn visible_cells(&self) -> VisibilityIter {
        self.visibility_grid.iter(self.world.count)
    }

    fn update_visibility(&mut self) {
        let &player_coord = self.world.entity_store.coord.get(&self.player_id).unwrap();
        let mut output_grid = VisibilityRefs {
            grid: &mut self.visibility_grid,
            world: &self.world,
        };
        self.shadowcast.observe(
            player_coord,
            &self.world.spatial_hash,
            128,
            self.world.count,
            &mut output_grid,
        );
    }

    pub fn player_active_meter_info(&self) -> ActiveMeterInfoIter {
        ActiveMeterInfoIter {
            entity_store: &self.world.entity_store,
            meter_metadata: self.active_meters.iter().enumerate(),
            entity_id: self.player_id,
            selected_meter: self.selected_meter,
        }
    }

    pub fn player_passive_meter_info(&self) -> PassiveMeterInfoIter {
        PassiveMeterInfoIter {
            entity_store: &self.world.entity_store,
            meter_metadata: self.passive_meters.iter(),
            entity_id: self.player_id,
        }
    }

    pub fn entity_store(&self) -> &EntityStore {
        &self.world.entity_store
    }
    pub fn spatial_hash(&self) -> &SpatialHashTable {
        &self.world.spatial_hash
    }

    pub fn goal_type(&self) -> GoalType {
        GoalType::Escape
    }

    pub fn overall_progress_meter(&self) -> Meter {
        Meter {
            value: (self.levels.len() - self.level_index) as i32 * 10,
            max: self.levels.len() as i32 * 10,
        }
    }

    fn player_turn(&mut self, input: Input) -> Option<Event> {
        match input {
            Input::Direction(direction) => {
                match self.selected_meter {
                    None => {
                        let current = *self.world.entity_store.coord.get(&self.player_id).unwrap();
                        let next = current + direction.coord();
                        self.messages
                            .changes
                            .push(insert::coord(self.player_id, next));
                    }
                    Some(ActiveMeterType::Gun) => {
                        let mut ammo = self.world
                            .entity_store
                            .gun_meter
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
                            prototypes::bullet(
                                bullet_id,
                                start_coord,
                                direction,
                                &mut self.messages,
                            );
                            common_animations::bullet(bullet_id, &mut self.messages);
                            ammo.value -= 1;
                            self.messages
                                .change(insert::gun_meter(self.player_id, ammo));
                        }
                    }
                    Some(ActiveMeterType::Medkit) => return None,
                }

                self.selected_meter = None;
            }
            Input::Enter => {
                match self.selected_meter {
                    None => return None,
                    Some(ActiveMeterType::Gun) => return None,
                    Some(ActiveMeterType::Medkit) => {
                        let mut medkit = self.world
                            .entity_store
                            .medkit_meter
                            .get(&self.player_id)
                            .cloned()
                            .unwrap();
                        if medkit.value > 0 {
                            let heal_amount = medkit.value;
                            medkit.value = 0;
                            self.messages
                                .change(insert::medkit_meter(self.player_id, medkit));

                            let mut health = self.world
                                .entity_store
                                .health_meter
                                .get(&self.player_id)
                                .cloned()
                                .unwrap();
                            health.value = ::std::cmp::min(health.value + heal_amount, health.max);
                            self.messages
                                .change(insert::health_meter(self.player_id, health));
                        }
                    }
                }
                self.selected_meter = None;
            }
            Input::ActiveMeterSelect(identifier) => {
                if let Some(meter_type) = self.active_meters.get(identifier.to_index()).cloned() {
                    self.selected_meter = Some(meter_type);
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
            &mut self.rng,
        );

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
            &mut self.messages,
            &mut self.swap_messages,
            &mut self.rng,
        )
    }

    fn process_turn_events(&mut self) -> Option<Event> {
        for entry in self.player_turn_events.iter_mut() {
            if entry.remaining == 0 {
                let change = match entry.event {
                    PlayerTurnEvent::ChangeActiveMeter(typ, change) => {
                        let mut meter =
                            Meter::from_entity_store(self.player_id, &self.world.entity_store, typ)
                                .expect("Missing meter for player turn event");
                        meter.value =
                            ::std::cmp::max(::std::cmp::min(meter.value + change, meter.max), 0);
                        typ.insert(self.player_id, meter)
                    }
                    PlayerTurnEvent::ChangePassiveMeter(typ, change) => {
                        let mut meter =
                            Meter::from_entity_store(self.player_id, &self.world.entity_store, typ)
                                .expect("Missing meter for player turn event");
                        meter.value =
                            ::std::cmp::max(::std::cmp::min(meter.value + change, meter.max), 0);
                        typ.insert(self.player_id, meter)
                    }
                };
                self.messages.changes.push(change);
                entry.remaining = entry.reset;
            } else {
                entry.remaining -= 1;
            }
        }

        let ret = self.change_context.process(
            &mut self.world,
            &mut self.messages,
            &mut self.swap_messages,
            &mut self.rng,
        );

        ret
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
                TurnState::Npcs => {
                    if let Some(event) = self.all_npc_turns() {
                        Some(event)
                    } else {
                        self.process_turn_events()
                    }
                }
            }
        } else {
            self.animation_tick(period)
        };

        self.update_visibility();

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
            active_meters,
            passive_meters,
            levels,
            level_index,
            player_turn_events,
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
            active_meters,
            passive_meters,
            selected_meter: None,
            levels,
            level_index,
            player_turn_events,
            shadowcast: ShadowcastContext::new(),
            visibility_grid: VisibilityGrid::new(size),
        }
    }
}
