use rand::Rng;
use entity_store::*;
use direction::CardinalDirection;
use common_animations;
use message_queues::PushMessages;
use meter::Meter;
use pickup::Pickup;
use direction::*;

pub fn precheck<'a, I: IntoIterator<Item = &'a EntityChange>>(
    changes: I,
    entity_store: &EntityStore,
    spatial_hash: &SpatialHashTable,
) -> bool {
    for change in changes {
        match change {
            &EntityChange::Insert(id, ComponentValue::Coord(coord)) => {
                if let Some(sh_cell) = spatial_hash.get(coord) {
                    let door_cell =
                        sh_cell.door_count > 0 && entity_store.door_opener.contains(&id);
                    let solid_cell = sh_cell.solid_count > 0 && !door_cell;
                    if solid_cell && entity_store.collider.contains(&id) {
                        return false;
                    }
                }
            }
            &EntityChange::Insert(..) => {}
            &EntityChange::Remove(..) => {}
        }
    }

    true
}

pub fn kevlar_blocks_attack<R: Rng>(
    entity_id: EntityId,
    entity_store: &EntityStore,
    rng: &mut R,
) -> Option<Meter> {
    if let Some(kevlar) = entity_store.kevlar_meter.get(&entity_id).cloned() {
        if kevlar.value > 0 {
            if rng.gen() {
                return Some(kevlar);
            }
        }
    }
    None
}

pub fn check<M, R>(
    change: &EntityChange,
    entity_store: &EntityStore,
    spatial_hash: &SpatialHashTable,
    id_allocator: &mut EntityIdAllocator,
    messages: &mut M,
    rng: &mut R,
) -> bool
where
    M: PushMessages,
    R: Rng,
{
    match change {
        &EntityChange::Insert(id, ComponentValue::Coord(coord)) => {
            if let Some(sh_cell) = spatial_hash.get(coord) {
                let dest_npc = sh_cell.npc_set.iter().next();

                if let Some(npc_id) = dest_npc {
                    if entity_store.player.contains(&id) {
                        if let Some(mut stamina) = entity_store.stamina_meter.get(&id).cloned() {
                            if stamina.value > 0 {
                                stamina.value -= 1;
                                messages.change(insert::stamina_meter(id, stamina));

                                let player_coord = entity_store
                                    .coord
                                    .get(&id)
                                    .cloned()
                                    .expect("Player missing coord");

                                for direction in CardinalDirections {
                                    let coord = player_coord + direction.coord();

                                    if let Some(sh_cell) = spatial_hash.get(coord) {
                                        if !sh_cell.npc_set.is_empty() {
                                            let punch_id = id_allocator.allocate();
                                            common_animations::punch(punch_id, coord, direction, messages);
                                        }
                                    }
                                }
                                messages.change(insert::stamina_tick(id, -1));
                            }
                        }
                    }
                    if entity_store.punch.contains(&id) {
                        if let Some(mut health) = entity_store.health_meter.get(&npc_id).cloned() {
                            health.value -= 1;
                            messages.change(insert::health_meter(*npc_id, health));
                            common_animations::damage_flash(*npc_id, messages);
                        }
                    }
                    if entity_store.bullet.contains_key(&id) {
                        if let Some(mut health) = entity_store.health_meter.get(&npc_id).cloned() {
                            health.value -= 1;
                            messages.change(insert::health_meter(*npc_id, health));
                            common_animations::damage_flash(*npc_id, messages);
                        }
                        messages.remove(id);
                        return false;
                    }
                    if entity_store.rail_gun_shot.contains(&id) {
                        if let Some(mut health) = entity_store.health_meter.get(&npc_id).cloned() {
                            health.value -= 1;
                            messages.change(insert::health_meter(*npc_id, health));
                            common_animations::rail_gun_damage_flash(*npc_id, messages);
                        }
                    }
                }

                let door_cell = sh_cell.door_count > 0 && entity_store.door_opener.contains(&id);

                let solid_cell =
                    (sh_cell.solid_count > 0 && !door_cell) || sh_cell.npc_set.len() > 0;

                if solid_cell && entity_store.collider.contains(&id) {
                    return false;
                }

                if let Some(&range) = entity_store.bullet.get(&id) {
                    if sh_cell.solid_count > 0 || range == 0 {
                        messages.remove(id);
                        return false;
                    }
                    messages.change(insert::bullet(id, range - 1));
                }

                let is_npc = entity_store.npc.contains_key(&id);

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

                    let player_id = entity_store.player.iter().next().unwrap();

                    let mut health = entity_store.health_meter.get(player_id).cloned().unwrap();

                    let change = if let Some(mut kevlar) =
                        kevlar_blocks_attack(*player_id, entity_store, rng)
                    {
                        kevlar.value -= 1;
                        insert::kevlar_meter(*player_id, kevlar)
                    } else {
                        health.value -= 1;
                        insert::health_meter(*player_id, health)
                    };

                    messages.change(change);

                    return false;
                };

                let is_player = entity_store.player.contains(&id);

                if is_player {
                    if sh_cell.stairs_count > 0 {
                        messages.ascend();
                    } else if sh_cell.exit_count > 0 {
                        messages.win();
                    } else {
                        if let Some(pickup_id) = sh_cell.pickup_set.iter().next() {
                            let &pickup = entity_store.pickup.get(pickup_id).unwrap();
                            match pickup {
                                Pickup::Ammo => {
                                    if let Some(mut ammo) = entity_store.gun_meter.get(&id).cloned()
                                    {
                                        ammo.value = ammo.max;
                                        messages.change(insert::gun_meter(id, ammo));
                                        messages.remove(*pickup_id);
                                    }
                                }
                                Pickup::RailGunAmmo => {
                                    if let Some(mut ammo) =
                                        entity_store.rail_gun_meter.get(&id).cloned()
                                    {
                                        ammo.value = ammo.max;
                                        messages.change(insert::rail_gun_meter(id, ammo));
                                        messages.remove(*pickup_id);
                                    }
                                }
                                Pickup::Health => {
                                    if let Some(mut health) =
                                        entity_store.health_meter.get(&id).cloned()
                                    {
                                        health.value = health.max;
                                        messages.change(insert::health_meter(id, health));
                                        messages.remove(*pickup_id);
                                    }
                                }
                                Pickup::Kevlar => {
                                    if let Some(mut kevlar) =
                                        entity_store.kevlar_meter.get(&id).cloned()
                                    {
                                        kevlar.value = kevlar.max;
                                        messages.change(insert::kevlar_meter(id, kevlar));
                                        messages.remove(*pickup_id);
                                    }
                                }
                            }
                        }
                        messages.move_player(coord);
                    }
                }
            }
        }
        &EntityChange::Insert(id, ComponentValue::HealthMeter(health)) => {
            if health.value == 1 {
                if let Some(mut tile_info) = entity_store.tile_info.get(&id).cloned() {
                    tile_info.wounded = true;
                    messages.change(insert::tile_info(id, tile_info));
                }
            } else if health.value == 0 {
                if entity_store.player.contains(&id) {
                    messages.lose();
                } else {
                    messages.remove(id);
                }
            } else {
                if let Some(health) = entity_store.health_meter.get(&id) {
                    if health.value == 1 {
                        if let Some(mut tile_info) = entity_store.tile_info.get(&id).cloned() {
                            tile_info.wounded = false;
                            messages.change(insert::tile_info(id, tile_info));
                        }
                    }
                }
            }
        }
        &EntityChange::Insert(id, ComponentValue::StaminaTick(value)) => {
            if value >= 1 {
                let mut stamina = *entity_store.stamina_meter.get(&id).unwrap();
                stamina.value = ::std::cmp::min(stamina.max, stamina.value + 1);
                messages.change(insert::stamina_meter(id, stamina));
                messages.change(insert::stamina_tick(id, 0));
            }
        }
        _ => (),
    }

    true
}
