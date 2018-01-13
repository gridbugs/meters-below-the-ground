use entity_store::*;

pub fn check(change:  &EntityChange,
             entity_store: &EntityStore,
             spatial_hash: &SpatialHashTable) -> bool {

    use self::EntityChange::*;
    use self::ComponentValue::*;
    match change {
        &Insert(id, Coord(coord)) => {
            if let Some(sh_cell) = spatial_hash.get(coord) {
                if sh_cell.solid_count > 0 && entity_store.collider.contains(&id) {
                    return false;
                }
            }
        },
        _ => (),
    }

    true
}
