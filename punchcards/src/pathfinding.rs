use grid_search::*;
use entity_store::*;
use direction::*;
use message_queues::PushMessages;

struct SpatialHashSolidCellGrid<'a>(&'a SpatialHashTable);
struct SpatialHashSolidOrOccupiedCellGrid<'a>(&'a SpatialHashTable);

impl<'a> SolidGrid for SpatialHashSolidCellGrid<'a> {
    fn is_solid(&self, coord: Coord) -> Option<bool> {
        self.0.get(coord).map(|cell| cell.solid_count > 0)
    }
}

impl<'a> SolidGrid for SpatialHashSolidOrOccupiedCellGrid<'a> {
    fn is_solid(&self, coord: Coord) -> Option<bool> {
        self.0
            .get(coord)
            .map(|cell| cell.solid_count > 0 || !cell.npc_set.is_empty())
    }
}

pub fn compute_player_map(
    player_coord: Coord,
    spatial_hash: &SpatialHashTable,
    bfs: &mut BfsContext,
    distance_map: &mut UniformDistanceMap<u32, DirectionsCardinal>,
) {
    bfs.populate_uniform_distance_map(
        &SpatialHashSolidCellGrid(spatial_hash),
        player_coord,
        Default::default(),
        distance_map,
    ).expect("Failed to compute player map");
}

pub fn act<M>(
    id: EntityId,
    entity_store: &EntityStore,
    spatial_hash: &SpatialHashTable,
    distance_map: &UniformDistanceMap<u32, DirectionsCardinal>,
    search: &mut SearchContext<u32>,
    path: &mut Vec<Direction>,
    messages: &mut M,
) where
    M: PushMessages,
{
    let coord = entity_store
        .coord
        .get(&id)
        .cloned()
        .expect("Entity missing coord");

    let cell = distance_map
        .get(coord)
        .cell()
        .expect("No distance cell for coord");

    let current_cost = cell.cost();

    assert!(current_cost > 0, "Unexpected 0 cost distance cell");

    const CONFIG: SearchConfig = SearchConfig {
        allow_solid_start: true,
    };

    let result = search.best_search_uniform_distance_map(
        &SpatialHashSolidOrOccupiedCellGrid(spatial_hash),
        coord,
        CONFIG,
        4, // max depth
        distance_map,
        path,
    );

    match result {
        Ok(_) => {
            if let Some(direction) = path.iter().next() {
                let delta = direction.coord();
                let new = coord + delta;
                messages.change(insert::coord(id, new));
            }
        }
        Err(Error::NoPath) => (),
        Err(e) => panic!("Unexpected pathfinding error: {:?}", e),
    }
}
