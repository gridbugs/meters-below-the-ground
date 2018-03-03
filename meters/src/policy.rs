use entity_store::*;
use tile_info;
use direction::CardinalDirection;
use common_animations;
use message_queues::PushMessages;

pub fn precheck<'a, I: IntoIterator<Item = &'a EntityChange>>(
    changes: I,
    entity_store: &EntityStore,
    spatial_hash: &SpatialHashTable,
) -> bool {
    use self::ComponentValue::*;
    use self::EntityChange::*;
    for change in changes {
        match change {
            &Insert(id, Coord(coord)) => {
                if let Some(sh_cell) = spatial_hash.get(coord) {
                    let solid_cell = sh_cell.solid_count > 0 || sh_cell.npc_set.len() > 0;
                    if solid_cell && entity_store.collider.contains(&id) {
                        return false;
                    }
                }
            }
            &Insert(..) => {}
            &Remove(..) => {}
        }
    }

    true
}

pub fn check<M>(
    change: &EntityChange,
    entity_store: &EntityStore,
    spatial_hash: &SpatialHashTable,
    id_allocator: &mut EntityIdAllocator,
    messages: &mut M,
) -> bool
where
    M: PushMessages,
{
    use self::EntityChange::*;
    use self::ComponentValue::*;
    match change {
        &Insert(id, Coord(coord)) => {
            if let Some(sh_cell) = spatial_hash.get(coord) {
                let dest_npc = sh_cell.npc_set.iter().next();

                if let Some(npc_id) = dest_npc {
                    if entity_store.punch.contains(&id) {
                        if let Some(hit_points) = entity_store.hit_points.get(&npc_id) {
                            messages.change(insert::hit_points(*npc_id, hit_points - 1));
                        }
                    }
                    if entity_store.bullet.contains(&id) {
                        if let Some(hit_points) = entity_store.hit_points.get(&npc_id) {
                            messages.change(insert::hit_points(*npc_id, hit_points - 1));
                        }
                        messages.remove(id);
                        return false;
                    }
                }

                let solid_cell = sh_cell.solid_count > 0 || sh_cell.npc_set.len() > 0;

                if solid_cell && entity_store.collider.contains(&id) {
                    return false;
                }

                if sh_cell.solid_count > 0 && entity_store.bullet.contains(&id) {
                    messages.remove(id);
                    return false;
                }

                let is_npc = entity_store.npc.contains(&id);

                if is_npc && dest_npc.is_some() {
                    // npcs can't move through one another
                    return false;
                }

                if sh_cell.player_count > 0 && is_npc {
                    let npc_coord = entity_store
                        .coord
                        .get(&id)
                        .cloned()
                        .expect("NPC missing coord");
                    let delta = coord - npc_coord;

                    let direction = CardinalDirection::from_unit_coord(delta);

                    let punch_id = id_allocator.allocate();

                    common_animations::punch(punch_id, coord, direction, messages);

                    return false;
                };

                let is_player = entity_store.player.contains(&id);

                if let Some(card_id) = sh_cell.card_set.iter().next() {
                    if is_player {
                        let card = entity_store.card.get(card_id).unwrap();
                        messages.take_card(*card_id, *card);
                    }
                }

                if is_player {
                    if sh_cell.stairs_count > 0 {
                        messages.next_level();
                    } else {
                        messages.move_player(coord);
                    }
                }
            }
        }
        &Insert(id, HitPoints(hit_points)) => {
            if hit_points == 0 {
                messages.remove(id);
            } else if hit_points == 1 {
                if let Some(tile_info) = entity_store.tile_info.get(&id) {
                    let tile_info = tile_info::TileInfo {
                        damaged: true,
                        ..*tile_info
                    };
                    messages.change(insert::tile_info(id, tile_info));
                }
            }
        }
        _ => (),
    }

    true
}
