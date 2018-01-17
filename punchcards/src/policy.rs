use append::Append;
use reaction::Reaction;
use entity_store::*;

pub fn check<A: Append<Reaction>>(change:  &EntityChange,
             entity_store: &EntityStore,
             spatial_hash: &SpatialHashTable,
             reactions: &mut A) -> bool {

    use self::EntityChange::*;
    use self::ComponentValue::*;
    match change {
        &Insert(id, Coord(coord)) => {
            if let Some(sh_cell) = spatial_hash.get(coord) {
                if sh_cell.solid_count > 0 && entity_store.collider.contains(&id) {
                    return false;
                }

                if let Some(card_id) = sh_cell.card_set.iter().next() {
                    let card = entity_store.card.get(card_id).unwrap();
                    reactions.append(Reaction::TakeCard(*card_id, *card));
                }
            }
        },
        _ => (),
    }

    true
}
