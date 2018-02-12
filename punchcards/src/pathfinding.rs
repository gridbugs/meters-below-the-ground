use grid_search::*;
use append::Append;
use entity_store::*;
use direction::*;

struct SpatialHashSolidCellGrid<'a>(&'a SpatialHashTable);

impl<'a> SolidGrid for SpatialHashSolidCellGrid<'a> {
    fn is_solid(&self, coord: Coord) -> Option<bool> {
        self.0.get(coord).map(|cell| cell.solid_count > 0)
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
        dijkstra_map,
    ).expect("Failed to compute player map");
}

pub fn act<Changes>(
    id: EntityId,
    entity_store: &EntityStore,
    dijkstra_map: &DijkstraMap<u32>,
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
        changes.append(insert::coord(id, new));
    }
}
