use append::Append;
use reaction::Reaction;
use entity_store::*;
use tile_info;

pub fn check<A: Append<Reaction>>(
    change: &EntityChange,
    entity_store: &EntityStore,
    spatial_hash: &SpatialHashTable,
    reactions: &mut A,
) -> bool {
    use self::EntityChange::*;
    use self::ComponentValue::*;
    match change {
        &Insert(id, Coord(coord)) => {
            if let Some(sh_cell) = spatial_hash.get(coord) {
                let dest_npc = sh_cell.npc_set.iter().next();

                if let Some(npc_id) = dest_npc {
                    if entity_store.punch.contains(&id) {
                        if let Some(hit_points) = entity_store.hit_points.get(&npc_id) {
                            reactions.append(Reaction::EntityChange(insert::hit_points(
                                *npc_id,
                                hit_points - 1,
                            )));
                        }
                    }
                }

                let solid_cell = sh_cell.solid_count > 0 || sh_cell.npc_set.len() > 0;

                if solid_cell && entity_store.collider.contains(&id) {
                    return false;
                }

                let is_npc = entity_store.npc.contains(&id);

                if is_npc && dest_npc.is_some() {
                    // npcs can't move through one another
                    return false;
                }

                if sh_cell.player_count > 0 {
                    // TODO damage player
                    return false;
                };

                let is_player = entity_store.player.contains(&id);

                if let Some(card_id) = sh_cell.card_set.iter().next() {
                    if is_player {
                        let card = entity_store.card.get(card_id).unwrap();
                        reactions.append(Reaction::TakeCard(*card_id, *card));
                    }
                }

                if is_player {
                    reactions.append(Reaction::PlayerMovedTo(coord));
                }
            }
        }
        &Insert(id, HitPoints(hit_points)) => {
            if hit_points == 0 {
                reactions.append(Reaction::RemoveEntity(id));
            } else if hit_points == 1 {
                if let Some(tile_info) = entity_store.tile_info.get(&id) {
                    let tile_info = tile_info::TileInfo {
                        damaged: true,
                        ..*tile_info
                    };
                    reactions.append(Reaction::EntityChange(insert::tile_info(id, tile_info)));
                }
            }
        }
        _ => (),
    }

    true
}
