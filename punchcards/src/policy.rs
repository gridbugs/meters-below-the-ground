use std::time::Duration;
use append::Append;
use reaction::*;
use entity_store::*;
use tile_info;
use direction::CardinalDirection;
use prototypes;
use timing;
use animation::*;

pub fn check<A: Append<Reaction> + Append<EntityChange>>(
    change: &EntityChange,
    entity_store: &EntityStore,
    spatial_hash: &SpatialHashTable,
    id_allocator: &mut EntityIdAllocator,
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

                if sh_cell.player_count > 0 && is_npc {
                    let npc_coord = entity_store
                        .coord
                        .get(&id)
                        .cloned()
                        .expect("NPC missing coord");
                    let delta = coord - npc_coord;
                    use grid_2d::Coord;

                    let direction = match delta {
                        Coord { x: 0, y: 1 } => CardinalDirection::South,
                        Coord { x: 0, y: -1 } => CardinalDirection::North,
                        Coord { x: 1, y: 0 } => CardinalDirection::East,
                        Coord { x: -1, y: 0 } => CardinalDirection::West,
                        _ => panic!("unexpected delta"),
                    };

                    let punch_id = id_allocator.allocate();

                    prototypes::punch(
                        punch_id,
                        coord,
                        direction,
                        reactions,
                    );

                    reactions.append(Reaction::StartAnimation(Animation::RemoveEntity(
                        punch_id,
                        Duration::from_millis(timing::PUNCH_MILLIS),
                    )));

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
