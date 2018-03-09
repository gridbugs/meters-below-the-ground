use rand::Rng;
use entity_store::*;
use grid_2d::Coord;
use meter::*;
use grid_search::*;
use direction::*;
use beacon::*;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum GoalType {
    Escape,
    KillEggs,
    KillBoss,
    ActivateBeacon,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GoalStateArgs {
    Escape { exit: Coord, player: Coord },
    KillEggs(Vec<EntityId>),
    KillBoss(EntityId),
    ActivateBeacon(EntityId),
}

struct SpatialHashSolidCellGrid<'a>(&'a SpatialHashTable);
impl<'a> SolidGrid for SpatialHashSolidCellGrid<'a> {
    fn is_solid(&self, coord: Coord) -> Option<bool> {
        self.0
            .get(coord)
            .map(|cell| cell.solid_count > 0 && cell.door_count == 0)
    }
}

impl GoalStateArgs {
    pub fn goal_state(self, spatial_hash: &SpatialHashTable) -> GoalState {
        match self {
            GoalStateArgs::Escape { player, exit } => {
                let mut bfs = BfsContext::new(spatial_hash.size());
                let mut distance_map =
                    UniformDistanceMap::new(spatial_hash.size(), DirectionsCardinal);
                bfs.populate_uniform_distance_map(
                    &SpatialHashSolidCellGrid(spatial_hash),
                    exit,
                    Default::default(),
                    &mut distance_map,
                ).expect("Failed to compute distance map");
                let initial = distance_map
                    .get(player)
                    .cell()
                    .expect("No path from player to exit")
                    .cost() as i32;
                GoalState::Escape {
                    distance_map,
                    initial,
                }
            }
            GoalStateArgs::KillEggs(coords) => GoalState::KillEggs(coords),
            GoalStateArgs::KillBoss(id) => GoalState::KillBoss(id),
            GoalStateArgs::ActivateBeacon(id) => GoalState::ActivateBeacon(id),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GoalState {
    Escape {
        distance_map: UniformDistanceMap<u32, DirectionsCardinal>,
        initial: i32,
    },
    KillEggs(Vec<EntityId>),
    KillBoss(EntityId),
    ActivateBeacon(EntityId),
}

const GOAL_TYPE_CHOICES: &[GoalType] = &[
    GoalType::KillEggs,
    GoalType::KillBoss,
    GoalType::ActivateBeacon,
];

pub fn choose_goal_type<R: Rng>(rng: &mut R) -> GoalType {
    let index = rng.gen::<usize>() % GOAL_TYPE_CHOICES.len();
    GOAL_TYPE_CHOICES[index]
}

impl GoalState {
    pub fn typ(&self) -> GoalType {
        match self {
            &GoalState::Escape { .. } => GoalType::Escape,
            &GoalState::KillEggs(_) => GoalType::KillEggs,
            &GoalState::KillBoss(_) => GoalType::KillBoss,
            &GoalState::ActivateBeacon(_) => GoalType::ActivateBeacon,
        }
    }
    pub fn with_goal_coords<F>(&self, entity_store: &EntityStore, mut f: F)
    where
        F: FnMut(Coord),
    {
        match self {
            &GoalState::Escape { .. } => {
                if let Some(stairs_id) = entity_store.stairs.iter().next() {
                    if let Some(&stairs_coord) = entity_store.coord.get(&stairs_id) {
                        f(stairs_coord);
                    }
                }
            }
            &GoalState::KillEggs(ref ids) => {
                for id in ids {
                    if entity_store.countdown_max.contains_key(id) {
                        if let Some(&coord) = entity_store.coord.get(&id) {
                            f(coord);
                        }
                    }
                }
            }
            &GoalState::KillBoss(id) => {
                if let Some(&coord) = entity_store.coord.get(&id) {
                    f(coord);
                }
            }
            &GoalState::ActivateBeacon(id) => {
                if let Some(&status) = entity_store.beacon.get(&id) {
                    if status == BeaconStatus::Inactive {
                        if let Some(&coord) = entity_store.coord.get(&id) {
                            f(coord);
                        }
                    }
                }
            }
        }
    }
    pub fn with_goal_meters<F>(&self, entity_store: &EntityStore, mut f: F)
    where
        F: FnMut(GoalMeterInfo),
    {
        match self {
            &GoalState::Escape {
                ref distance_map,
                initial,
            } => {
                let player_id = entity_store.player.iter().next().unwrap();
                let player_coord = entity_store.coord.get(player_id).unwrap();
                if let Some(cell) = distance_map.get(*player_coord).cell() {
                    f(GoalMeterInfo {
                        typ: GoalMeterType::DistanceToExit,
                        meter: Meter::new(cell.cost() as i32, initial),
                    })
                }
            }
            &GoalState::KillEggs(ref ids) => {
                for id in ids {
                    if let Some(countdown) = entity_store.countdown.get(id).cloned() {
                        if let Some(countdown_max) = entity_store.countdown_max.get(id).cloned() {
                            f(GoalMeterInfo {
                                typ: GoalMeterType::SuperEggHealth,
                                meter: Meter {
                                    value: countdown,
                                    max: countdown_max,
                                },
                            })
                        }
                    }
                }
            }
            &GoalState::KillBoss(id) => {
                if let Some(health) = entity_store.health_meter.get(&id).cloned() {
                    f(GoalMeterInfo {
                        typ: GoalMeterType::BossHealth,
                        meter: health,
                    })
                }
            }
            &GoalState::ActivateBeacon(_) => (),
        }
    }
    pub fn is_complete(&self, entity_store: &EntityStore) -> bool {
        match self {
            &GoalState::Escape { .. } => false,
            &GoalState::KillEggs(ref ids) => {
                for id in ids {
                    if entity_store.countdown_max.contains_key(id) {
                        return false;
                    }
                }
                true
            }
            &GoalState::KillBoss(id) => !entity_store.health_meter.contains_key(&id),
            &GoalState::ActivateBeacon(id) => {
                if let Some(&status) = entity_store.beacon.get(&id) {
                    status == BeaconStatus::Active
                } else {
                    false
                }
            }
        }
    }
}

#[derive(Clone, Debug, Copy)]
pub enum GoalMeterType {
    BossHealth,
    DistanceToExit,
    SuperEggHealth,
}

#[derive(Clone, Debug)]
pub struct GoalMeterInfo {
    pub typ: GoalMeterType,
    pub meter: Meter,
}
