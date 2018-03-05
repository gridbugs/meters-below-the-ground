use std::cmp::Ordering;
use grid_search::*;
use entity_store::*;
use direction::*;
use message_queues::PushMessages;

pub struct PathfindingConfig {
    pub open_doors: bool,
}

struct SpatialHashSolidCellGrid<'a> {
    grid: &'a SpatialHashTable,
    open_doors: bool,
}
struct SpatialHashSolidOrOccupiedCellGrid<'a> {
    grid: &'a SpatialHashTable,
    open_doors: bool,
}

impl<'a> SolidGrid for SpatialHashSolidCellGrid<'a> {
    fn is_solid(&self, coord: Coord) -> Option<bool> {
        self.grid.get(coord).map(|cell| cell.solid_count > 0 && (self.open_doors || cell.door_count == 0))
    }
}

impl<'a> SolidGrid for SpatialHashSolidOrOccupiedCellGrid<'a> {
    fn is_solid(&self, coord: Coord) -> Option<bool> {
        self.grid
            .get(coord)
            .map(|cell| (cell.solid_count > 0 && (self.open_doors || cell.door_count == 0)) || !cell.npc_set.is_empty())
    }
}

#[derive(Debug, Clone)]
pub struct PathfindingContext {
    search: SearchContext<u32>,
    bfs: BfsContext,
    distance_map: UniformDistanceMap<u32, DirectionsCardinal>,
    distance_map_open_doors: UniformDistanceMap<u32, DirectionsCardinal>,
    path: Vec<Direction>,
}

impl PathfindingContext {
    pub fn new(size: Size) -> Self {
        Self {
            search: SearchContext::new(size),
            bfs: BfsContext::new(size),
            distance_map: UniformDistanceMap::new(size, DirectionsCardinal),
            distance_map_open_doors: UniformDistanceMap::new(size, DirectionsCardinal),
            path: Vec::new(),
        }
    }

    pub fn update_player_map(&mut self, player_coord: Coord, spatial_hash: &SpatialHashTable) {
        self.bfs
            .populate_uniform_distance_map(
                &SpatialHashSolidCellGrid {
                    grid: spatial_hash,
                    open_doors: false,
                },
                player_coord,
                Default::default(),
                &mut self.distance_map,
            )
            .expect("Failed to update player distance map");
        self.bfs
            .populate_uniform_distance_map(
                &SpatialHashSolidCellGrid {
                    grid: spatial_hash,
                    open_doors: true,
                },
                player_coord,
                Default::default(),
                &mut self.distance_map_open_doors,
            )
            .expect("Failed to update player distance map");

    }

    pub fn act<M>(
        &mut self,
        id: EntityId,
        entity_store: &EntityStore,
        spatial_hash: &SpatialHashTable,
        config: PathfindingConfig,
        messages: &mut M,
    ) where
        M: PushMessages,
    {
        let coord = entity_store
            .coord
            .get(&id)
            .cloned()
            .expect("Entity missing coord");

        let cell = if let Some(cell) =self.distance_map
            .get(coord)
            .cell()
        {
            cell
        } else {
            // no path to player
            return;
        };

        let current_cost = cell.cost();

        assert!(current_cost > 0, "Unexpected 0 cost distance cell");

        const CONFIG: SearchConfig = SearchConfig {
            allow_solid_start: true,
        };

        let distance_map = if config.open_doors {
            &self.distance_map_open_doors
        } else {
            &self.distance_map
        };

        let result = self.search.best_search_uniform_distance_map(
            &SpatialHashSolidOrOccupiedCellGrid {
                grid: spatial_hash,
                open_doors: config.open_doors,
            },
            coord,
            CONFIG,
            4, // max depth
            distance_map,
            &mut self.path,
        );

        match result {
            Ok(_) => {
                if let Some(direction) = self.path.iter().next() {
                    let delta = direction.coord();
                    let new = coord + delta;
                    messages.change(insert::coord(id, new));
                }
            }
            Err(Error::NoPath) => (),
            Err(e) => panic!("Unexpected pathfinding error: {:?}", e),
        }
    }

    pub fn sort_entities_by_distance_to_player(
        &self,
        entity_store: &EntityStore,
        entities: &mut Vec<EntityId>,
    ) {
        let coord = &entity_store.coord;
        let distance_map = &self.distance_map_open_doors;
        entities.sort_by(|a, b| {
            let coord_a = coord.get(a).expect("Entity missing coord");
            let coord_b = coord.get(b).expect("Entity missing coord");
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
}
