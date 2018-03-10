use rand::Rng;
use entity_store::*;
use direction::CardinalDirection;
use common_animations;
use message_queues::PushMessages;
use meter::Meter;
use pickup::Pickup;
use direction::*;
use alert::*;
use beacon::*;
use tile::*;
use pushed::*;

pub fn precheck<'a, I: IntoIterator<Item = &'a EntityChange>>(
    changes: I,
    entity_store: &EntityStore,
    spatial_hash: &SpatialHashTable,
) -> Result<(), Option<Alert>> {
    for change in changes {
        match change {
            &EntityChange::Insert(id, ComponentValue::Coord(coord)) => {
                if !entity_store.player.contains(&id) {
                    continue;
                }
                if let Some(sh_cell) = spatial_hash.get(coord) {
                    let door_cell =
                        sh_cell.door_count > 0 && entity_store.door_opener.contains(&id);
                    let solid_cell = sh_cell.solid_count > 0 && !door_cell;
                    if solid_cell && entity_store.collider.contains(&id) {
                        return Err(None);
                    }

                    if !sh_cell.npc_set.is_empty() {
                        if let Some(stamina) = entity_store.stamina_meter.get(&id) {
                            if stamina.value == 0 {
                                return Err(Some(Alert::NoStamina));
                            }
                        } else {
                            return Err(Some(Alert::NoStamina));
                        }
                    }
                }
            }
            &EntityChange::Insert(..) => {}
            &EntityChange::Remove(..) => {}
        }
    }

    Ok(())
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

                if let Some(mut wave) = entity_store.metabol_wave.get(&id).cloned() {
                    if wave.range == 0 {
                        return false;
                    }

                    let next_id = id_allocator.allocate();
                    if wave.leader {
                        common_animations::metabol_wave(
                            next_id,
                            coord + wave.direction.coord(),
                            true,
                            false,
                            false,
                            wave.direction,
                            wave.range - 1,
                            messages,
                        );

                        if wave.left {
                            common_animations::metabol_wave(
                                id_allocator.allocate(),
                                coord + wave.direction.left45(),
                                true,
                                true,
                                false,
                                wave.direction,
                                wave.range - 1,
                                messages,
                                );
                            common_animations::metabol_wave(
                                id_allocator.allocate(),
                                coord + wave.direction.left90(),
                                false,
                                false,
                                false,
                                wave.direction,
                                wave.range - 1,
                                messages,
                                );

                        }
                        if wave.right {
                            common_animations::metabol_wave(
                                id_allocator.allocate(),
                                coord + wave.direction.right45(),
                                true,
                                false,
                                true,
                                wave.direction,
                                wave.range - 1,
                                messages,
                                );
                            common_animations::metabol_wave(
                                id_allocator.allocate(),
                                coord + wave.direction.right90(),
                                false,
                                false,
                                false,
                                wave.direction,
                                wave.range - 1,
                                messages,
                                );

                        }
                    }
                }

                if let Some(mut wave) = entity_store.push_wave.get(&id).cloned() {
                    if sh_cell.solid_count > 0 && sh_cell.door_count == 0 {
                        return false;
                    }
                    if wave.range == 0 {
                        return false;
                    }

                    let next_id = id_allocator.allocate();
                    if wave.leader {
                        common_animations::push_wave(
                            next_id,
                            coord + wave.direction.coord(),
                            true,
                            false,
                            false,
                            wave.direction,
                            wave.range - 1,
                            messages,
                        );

                        if wave.left {
                            common_animations::push_wave(
                                id_allocator.allocate(),
                                coord + wave.direction.left45(),
                                true,
                                true,
                                false,
                                wave.direction,
                                wave.range - 1,
                                messages,
                                );
                            common_animations::push_wave(
                                id_allocator.allocate(),
                                coord + wave.direction.left90(),
                                false,
                                false,
                                false,
                                wave.direction,
                                wave.range - 1,
                                messages,
                                );

                        }
                        if wave.right {
                            common_animations::push_wave(
                                id_allocator.allocate(),
                                coord + wave.direction.right45(),
                                true,
                                false,
                                true,
                                wave.direction,
                                wave.range - 1,
                                messages,
                                );
                            common_animations::push_wave(
                                id_allocator.allocate(),
                                coord + wave.direction.right90(),
                                false,
                                false,
                                false,
                                wave.direction,
                                wave.range - 1,
                                messages,
                                );

                        }
                    }

                }

                if let Some(npc_id) = dest_npc {
                    if entity_store.metabol_wave.contains_key(&id) {
                        if let Some(countdown) = entity_store.countdown.get(npc_id).cloned() {
                            messages.change(insert::delayed_transform(*npc_id));
                            messages.change(insert::countdown(*npc_id, countdown + 20));
                            if let Some(mut tile_info) = entity_store.tile_info.get(npc_id).cloned() {
                                tile_info.delayed_transform = true;
                                messages.change(insert::tile_info(*npc_id, tile_info));
                            }
                        }
                    }

                    if let Some(wave) = entity_store.push_wave.get(&id) {

                        let player_id = entity_store.player.iter().next().unwrap();
                        let player_coord = entity_store.coord.get(player_id).cloned().unwrap();

                        if let Some(npc_coord) = entity_store.coord.get(&npc_id).cloned() {

                            let delta = npc_coord - player_coord;

                            let direction = if delta.x.abs() > delta.y.abs() {
                                if delta.x > 0 {
                                    CardinalDirection::East
                                } else {
                                    CardinalDirection::West
                                }
                            } else {
                                if delta.y > 0 {
                                    CardinalDirection::South
                                } else {
                                    CardinalDirection::North
                                }
                            };

                            let pushed = PushedInfo {
                                direction,
                                distance: wave.range,
                                range: 2,
                            };
                            messages.change(insert::pushed(*npc_id, pushed));
                            if let Some(mut tile_info) = entity_store.tile_info.get(npc_id).cloned() {
                                tile_info.pushed = true;
                                messages.change(insert::tile_info(*npc_id, tile_info));
                            }
                        }
                    }

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
                                            common_animations::punch(
                                                punch_id,
                                                coord,
                                                direction,
                                                messages,
                                            );
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

                let door_cell = sh_cell.door_count > 0;

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
                        messages.alert(Alert::ArmourBlock);
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
                    } else if let Some(beacon_id) = sh_cell.beacon_set.iter().next() {
                        messages.change(insert::beacon(*beacon_id, BeaconStatus::Active));
                        messages.alert(Alert::BeaconActive);
                        for (&id, &info) in entity_store.npc.iter() {
                            let mut info = info;
                            info.active = true;
                            messages.change(insert::npc(id, info));
                        }
                        return false;
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
                                Pickup::MetabolAmmo => {
                                    if let Some(mut ammo) =
                                        entity_store.metabol_meter.get(&id).cloned()
                                    {
                                        ammo.value = ammo.max;
                                        messages.change(insert::metabol_meter(id, ammo));
                                        messages.remove(*pickup_id);
                                    }
                                }
                                Pickup::PushAmmo => {
                                    if let Some(mut ammo) =
                                        entity_store.push_meter.get(&id).cloned()
                                    {
                                        ammo.value = ammo.max;
                                        messages.change(insert::push_meter(id, ammo));
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
            if let Some(mut tile_info) = entity_store.tile_info.get(&id).cloned() {
                tile_info.health_meter = Some(health);
                messages.change(insert::tile_info(id, tile_info));
            }

            if health.value == 0 {
                if entity_store.player.contains(&id) {
                    messages.lose();
                } else {
                    messages.remove(id);
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
        &EntityChange::Insert(id, ComponentValue::Beacon(BeaconStatus::Active)) => {
            if let Some(mut tile_info) = entity_store.tile_info.get(&id).cloned() {
                tile_info.tile = Tile::BeaconActive;
                messages.change(insert::tile_info(id, tile_info));
            }
        }
        _ => (),
    }

    true
}
