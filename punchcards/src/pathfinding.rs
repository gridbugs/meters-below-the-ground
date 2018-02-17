use grid_search::*;
use append::Append;
use entity_store::*;
use direction::*;
use best::*;
use invert::*;

struct SpatialHashSolidCellGrid<'a>(&'a SpatialHashTable);
struct SpatialHashSolidOrOccupiedCellGrid<'a>(&'a SpatialHashTable);

impl<'a> SolidGrid for SpatialHashSolidCellGrid<'a> {
    fn is_solid(&self, coord: Coord) -> Option<bool> {
        self.0.get(coord).map(|cell| cell.solid_count > 0)
    }
}

impl<'a> SolidGrid for SpatialHashSolidOrOccupiedCellGrid<'a> {
    fn is_solid(&self, coord: Coord) -> Option<bool> {
        self.0.get(coord).map(|cell| {
            cell.solid_count > 0 ||
                !cell.npc_set.is_empty()
        })
    }
}

pub fn compute_player_map(
    player_coord: Coord,
    spatial_hash: &SpatialHashTable,
    bfs: &mut BfsContext,
    dijkstra_map: &mut DijkstraMap<u32>,
) {
    bfs.populate_dijkstra_map(
        &SpatialHashSolidCellGrid(spatial_hash),
        player_coord,
        DirectionsCardinal,
        Default::default(),
        dijkstra_map,
    ).expect("Failed to compute player map");
}

pub fn act<Changes>(
    id: EntityId,
    entity_store: &EntityStore,
    spatial_hash: &SpatialHashTable,
    dijkstra_map: &DijkstraMap<u32>,
    bfs: &mut BfsContext,
    path: &mut Vec<Direction>,
    changes: &mut Changes,
) where
    Changes: Append<EntityChange>,
{
    let coord = entity_store
        .coord
        .get(&id)
        .cloned()
        .expect("Entity missing coord");

    let cell = dijkstra_map.get(coord).cell()
        .expect("No dijkstra cell for coord");

    let current_cost = cell.cost();

    assert!(current_cost > 0, "Unexpected 0 cost dijkstra cell");

    if current_cost == 1 {
        return;
    }

    let delta = cell.direction().coord();
    let new = coord + delta;
    let sh_cell = spatial_hash.get(new).expect("Coord outside spatial hash");
    if sh_cell.npc_set.is_empty() {
        changes.append(insert::coord(id, new));
    } else {

        let mut best = BestMapNonEmpty::new(partial_invert(cell.cost()), coord);

        for direction in DirectionsCardinal {
            let neighbour_coord = coord + direction.coord();
            if let DijkstraMapEntry::Cell(neighbour) = dijkstra_map.get(neighbour_coord) {
                let sh_cell = spatial_hash.get(neighbour_coord).expect("Coord outside spatial hash");
                if sh_cell.npc_set.is_empty() {
                    best.insert_gt(partial_invert(neighbour.cost()), neighbour_coord);
                }
            }
        }

        let best_coord = best.into_value();
        if best_coord != coord {
            changes.append(insert::coord(id, best_coord));
            return;
        }
    }

    // We're in a local minima of the dijkstra map.
    // Let's look for a path to a nearby cell whose dijkstra map value
    // is less than ours.

    const CONFIG: BfsConfig = BfsConfig {
        allow_solid_start: true,
        max_depth: 4,
    };

    let score = move |coord| {
        dijkstra_map.get(coord).cell().map(|cell| {
            partial_invert(cell.cost())
        })
    };

    let result = bfs.bfs_best(
        &SpatialHashSolidOrOccupiedCellGrid(spatial_hash),
        coord,
        score,
        DirectionsCardinal,
        CONFIG,
        path,
    );

    match result {
        Ok(_) => {
            if let Some(direction) = path.iter().next() {
                let delta = direction.coord();
                let new = coord + delta;
                changes.append(insert::coord(id, new));
            }
        }
        Err(Error::NoPath) => (),
        Err(e) => panic!("Unexpected pathfinding error: {:?}", e)
    }
}
