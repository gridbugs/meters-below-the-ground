use std::time::Duration;
use std::cmp::Ordering;
use std::collections::HashSet;
use grid_2d::{Coord, Size};
use grid_search::*;
use entity_store::*;
use input::Input;
use policy;
use prototypes;
use card::*;
use card_state::*;
use tile::*;
use reaction::*;
use animation::*;
use rand::{Rng, SeedableRng, StdRng};
use append::Append;
use direction::{Direction, DirectionsCardinal};
use pathfinding;

const INITIAL_HAND_SIZE: usize = 4;

pub enum Meta {
    GameOver,
}

#[derive(Clone, Debug)]
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
    game_state: GameState,
    player_id: EntityId,
    changes: Vec<EntityChange>,
    reactions: Vec<Reaction>,
    animations: Vec<Animation>,
    card_state: CardState,
    input_state: InputState,
    search_context: SearchContext<u32>,
    bfs_context: BfsContext,
    distance_map: UniformDistanceMap<u32, DirectionsCardinal>,
    rng: StdRng,
    turn: TurnState,
    recompute_player_map: Option<Coord>,
    path: Vec<Direction>,
    npc_order: Vec<EntityId>,
    seen_animation_channels: HashSet<AnimationChannel>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaveState {
    changes: Vec<EntityChange>,
    id_allocator: EntityIdAllocator,
    count: u64,
    player_id: EntityId,
    card_state: CardState,
    animations: Vec<Animation>,
    input_state: InputState,
    next_rng_seed: usize,
    size: Size,
    turn: TurnState,
    recompute_player_map: Option<Coord>,
}

impl State {
    pub fn new(rng_seed: usize) -> Self {
        let mut rng = StdRng::from_seed(&[rng_seed]);

        let strings = vec![
            "##########",
            "#@...1111#",
            "#.....#11#",
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
        let mut changes = Vec::new();
        let animations = Vec::new();
        let mut player_id = None;
        let mut player_coord = None;

        for (y, line) in strings.iter().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                let coord = Coord::new(x as i32, y as i32);
                match ch {
                    '#' => {
                        prototypes::wall(id_allocator.allocate(), coord, &mut changes);
                        prototypes::floor(id_allocator.allocate(), coord, &mut changes);
                    }
                    '.' => {
                        prototypes::floor(id_allocator.allocate(), coord, &mut changes);
                    }
                    'm' => {
                        prototypes::card(
                            id_allocator.allocate(),
                            coord,
                            Card::Move,
                            Tile::CardMove,
                            &mut changes,
                        );
                        prototypes::floor(id_allocator.allocate(), coord, &mut changes);
                    }
                    '0' => {
                        prototypes::target_dummy(id_allocator.allocate(), coord, &mut changes);
                        prototypes::floor(id_allocator.allocate(), coord, &mut changes);
                    }
                    '1' => {
                        prototypes::small_robot(id_allocator.allocate(), coord, &mut changes);
                        prototypes::floor(id_allocator.allocate(), coord, &mut changes);
                    }
                    '@' => {
                        let id = id_allocator.allocate();
                        player_id = Some(id);
                        player_coord = Some(coord);
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
            game_state: GameState {
                entity_store,
                spatial_hash,
                entity_components,
                id_allocator,
                count: 0,
            },
            input_state: InputState::WaitingForCardSelection,
            player_id,
            changes,
            animations,
            reactions: Vec::new(),
            card_state,
            search_context: SearchContext::new(size),
            bfs_context: BfsContext::new(size),
            distance_map: UniformDistanceMap::new(size, DirectionsCardinal),
            rng,
            turn: TurnState::Player,
            recompute_player_map: player_coord,
            path: Vec::new(),
            npc_order: Vec::new(),
            seen_animation_channels: HashSet::new(),
        }
    }

    pub fn from_save_state(
        SaveState {
            mut changes,
            id_allocator,
            count,
            player_id,
            card_state,
            animations,
            input_state,
            next_rng_seed,
            size,
            turn,
            recompute_player_map,
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
            game_state: GameState {
                entity_store,
                spatial_hash,
                entity_components,
                id_allocator,
                count,
            },
            input_state,
            player_id,
            changes: Vec::new(),
            animations,
            reactions: Vec::new(),
            card_state,
            search_context: SearchContext::new(size),
            bfs_context: BfsContext::new(size),
            distance_map: UniformDistanceMap::new(size, DirectionsCardinal),
            rng: StdRng::from_seed(&[next_rng_seed]),
            turn,
            recompute_player_map,
            path: Vec::new(),
            npc_order: Vec::new(),
            seen_animation_channels: HashSet::new(),
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
            animations: self.animations.clone(),
            input_state: self.input_state.clone(),
            next_rng_seed,
            size: Size::new(
                self.game_state.spatial_hash.width(),
                self.game_state.spatial_hash.height(),
            ),
            turn: self.turn,
            recompute_player_map: self.recompute_player_map,
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
        if self.animations.is_empty() {
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
                                        &mut self.changes,
                                        &mut self.reactions,
                                    );
                                    if policy::precheck(&self.changes, &self.game_state.entity_store, 
                                                        &self.game_state.spatial_hash) {

                                        let card_to_check = self.card_state.hand.remove_card(index);
                                        assert_eq!(card, card_to_check);
                                        self.card_state.fill_hand();
                                        self.input_state = InputState::WaitingForCardSelection;
                                        self.turn = TurnState::Npcs;

                                        let ret = process_changes(
                                            &mut self.game_state,
                                            &mut self.card_state,
                                            &mut self.changes,
                                            &mut self.reactions,
                                            &mut self.animations,
                                            &mut self.recompute_player_map,
                                            &mut self.rng,
                                        );

                                        self.game_state.count += 1;

                                        ret
                                    } else {
                                        self.changes.clear();
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
                        pathfinding::compute_player_map(
                            player_coord,
                            &self.game_state.spatial_hash,
                            &mut self.bfs_context,
                            &mut self.distance_map,
                        );
                    }

                    self.npc_order.clear();
                    for &id in self.game_state.entity_store.npc.iter() {
                        self.npc_order.push(id);
                    }

                    {
                        let coord = &self.game_state.entity_store.coord;
                        let distance_map = &self.distance_map;
                        self.npc_order.sort_by(|a, b| {
                            let coord_a = coord.get(a).expect("NPC missing coord");
                            let coord_b = coord.get(b).expect("NPC missing coord");
                            if let Some(cell_a) = distance_map.get(*coord_a).cell() {
                                if let Some(cell_b) = distance_map.get(*coord_b).cell() {
                                    cell_a.cost().cmp(&cell_b.cost())
                                } else {
                                    Ordering::Less
                                }
                            } else {
                                if distance_map.get(*coord_b).cell().is_some() {
                                    Ordering::Greater
                                } else {
                                    Ordering::Equal
                                }
                            }
                        });
                    }

                    for &id in self.npc_order.iter() {
                        pathfinding::act(
                            id,
                            &self.game_state.entity_store,
                            &self.game_state.spatial_hash,
                            &self.distance_map,
                            &mut self.search_context,
                            &mut self.path,
                            &mut self.changes,
                        );
                        if let Some(meta) = process_changes(
                            &mut self.game_state,
                            &mut self.card_state,
                            &mut self.changes,
                            &mut self.reactions,
                            &mut self.animations,
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
            for animation in self.animations.drain(..) {
                let channel = animation.channel;
                if self.seen_animation_channels.contains(&channel) {
                    self.reactions.push(Reaction::StartAnimation(animation));
                } else {
                    match animation.step(period, &mut self.reactions) {
                        AnimationStatus::Continuing => { self.seen_animation_channels.insert(channel); }
                        AnimationStatus::Finished => (),
                    }
                }
            }
            process_changes(
                &mut self.game_state,
                &mut self.card_state,
                &mut self.changes,
                &mut self.reactions,
                &mut self.animations,
                &mut self.recompute_player_map,
                &mut self.rng,
            )
        }
    }
}

fn process_changes<R: Rng>(
    game_state: &mut GameState,
    card_state: &mut CardState,
    changes: &mut Vec<EntityChange>,
    reactions: &mut Vec<Reaction>,
    animations: &mut Vec<Animation>,
    recompute_player_map: &mut Option<Coord>,
    rng: &mut R,
) -> Option<Meta> {
    loop {
        for change in changes.drain(..) {
            if !policy::check(
                &change,
                &game_state.entity_store,
                &game_state.spatial_hash,
                &mut game_state.id_allocator,
                reactions,
            ) {
                continue;
            }

            game_state
                .spatial_hash
                .update(&game_state.entity_store, &change, game_state.count);
            game_state.entity_components.update(&change);
            game_state.entity_store.commit(change);
        }

        if reactions.is_empty() {
            if card_state.hand.is_empty() {
                break Some(Meta::GameOver);
            } else {
                break None;
            }
        } else {
            for reaction in reactions.drain(..) {
                match reaction {
                    Reaction::TakeCard(entity_id, card) => {
                        card_state.deck.add_random(card, rng);
                        game_state.delete_entity(entity_id, changes);
                    }
                    Reaction::RemoveEntity(entity_id) => {
                        game_state.delete_entity(entity_id, changes);
                        game_state.id_allocator.free(entity_id);
                    }
                    Reaction::StartAnimation(animation) => {
                        animations.push(animation);
                    }
                    Reaction::EntityChange(change) => {
                        changes.push(change);
                    }
                    Reaction::PlayerMovedTo(coord) => {
                        *recompute_player_map = Some(coord);
                    }
                }
            }
        }
    }
}
