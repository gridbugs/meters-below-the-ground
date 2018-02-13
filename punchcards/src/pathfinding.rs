use grid_search::*;
use append::Append;
use entity_store::*;
use direction::*;
use best::*;
use invert::*;

struct SpatialHashSolidCellGrid<'a>(&'a SpatialHashTable);
struct SpatialHashSolidOrOccupiedCellGrid<'a>(&'a SpatialHashTable);

const WAIT_SPREAD: u32 = 4;

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
    player_id: EntityId,
    entity_store: &EntityStore,
    spatial_hash: &SpatialHashTable,
    dijkstra_map: &DijkstraMap<u32>,
    search: &mut SearchContext<u32>,
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
    if let DijkstraMapEntry::Cell(cell) = dijkstra_map.get(coord) {
        let delta = cell.direction().coord();
        let new = coord + delta;
        let sh_cell = spatial_hash.get(new).expect("Coord outside spatial hash");
        if sh_cell.npc_set.is_empty() {
            changes.append(insert::coord(id, new));
        } else {

            let optimal_cost = cell.cost();

            let mut best = BestMapNonEmpty::new(Invert::new(optimal_cost), coord);

            for direction in DirectionsCardinal {
                let neighbour_coord = coord + direction.coord();
                if let DijkstraMapEntry::Cell(neighbour) = dijkstra_map.get(neighbour_coord) {
                    let sh_cell = spatial_hash.get(neighbour_coord).expect("Coord outside spatial hash");
                    if sh_cell.npc_set.is_empty() {
                        best.insert_gt(Invert::new(neighbour.cost()), neighbour_coord);
                    }
                }
            }

            let best_coord = best.into_value();
            if best_coord != coord {
                changes.append(insert::coord(id, best_coord));
            } else {

                let player_coord = entity_store
                    .coord
                    .get(&player_id)
                    .cloned()
                    .expect("Player missing coord");
                let result = search.jump_point_search_cardinal_manhatten_distance_heuristic(
                    &SpatialHashSolidOrOccupiedCellGrid(spatial_hash),
                    coord,
                    player_coord,
                    Default::default(),
                    path,
                    );

                match result {
                    Ok(metadata) => {

                        let current_cost = metadata.cost;

                        if current_cost < optimal_cost + WAIT_SPREAD {
                            if let Some(direction) = path.iter().next() {
                                let delta = direction.coord();
                                let new = coord + delta;
                                changes.append(insert::coord(id, new));
                            }
                        }
                    }
                    Err(Error::NoPath) => {

                        let result = search.jump_point_search_cardinal_manhatten_distance_heuristic(
                            &SpatialHashSolidCellGrid(spatial_hash),
                            coord,
                            player_coord,
                            Default::default(),
                            path,
                            );

                        if result.is_ok() {
                            if let Some(direction) = path.iter().next() {
                                let delta = direction.coord();
                                let new = coord + delta;
                                changes.append(insert::coord(id, new));
                            }
                        }
                    }
                    other => panic!("Pathfinding failed: {:?}", other),
                }
            }
        }
    }
}
