use rand::Rng;
use direction::CardinalDirection;
use entity_store::*;
use grid_2d::Coord;
use tile::Tile;
use tile_info::TileInfo;
use message_queues::PushMessages;
use meter::*;
use pickup::Pickup;
use npc_info::*;
use transform::*;

const FLOOR_DEPTH: i32 = 1;
const WALL_DEPTH: i32 = 2;
const STAIRS_DEPTH: i32 = 3;
const PICKUP_DEPTH: i32 = 4;
const BULLET_DEPTH: i32 = 5;
const NPC_DEPTH: i32 = 6;
const RAIL_GUN_SHOT_DEPTH: i32 = 7;
const PLAYER_DEPTH: i32 = 8;
const ANIMATION_DEPTH: i32 = 9;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Prototype {
    Punch(EntityId, Coord, CardinalDirection),
    RailGunShot(EntityId, Coord, CardinalDirection),
}

impl Prototype {
    pub fn instantiate<M: PushMessages>(self, messages: &mut M) -> EntityId {
        match self {
            Prototype::Punch(id, coord, direction) => {
                punch(id, coord, direction, messages);
                id
            }
            Prototype::RailGunShot(id, coord, direction) => {
                rail_gun_shot(id, coord, direction, messages);
                id
            }
        }
    }
}

pub fn player<M: PushMessages>(id: EntityId, coord: Coord, messages: &mut M) {
    messages.change(insert::coord(id, coord));
    messages.change(insert::player(id));
    messages.change(insert::door_opener(id));
    messages.change(insert::collider(id));
    let health = Meter::full(MeterType::Health.player_max());
    messages.change(insert::tile_info(
        id,
        TileInfo::with_health(Tile::Player, PLAYER_DEPTH, health),
    ));
    messages.change(insert::stamina_meter(
        id,
        Meter::new(
            MeterType::Stamina.player_max() / 2,
            MeterType::Stamina.player_max(),
        ),
    ));
    messages.change(insert::stamina_tick(id, 0));
    messages.change(insert::health_meter(id, health));
}

pub fn floor<M: PushMessages>(id: EntityId, coord: Coord, messages: &mut M) {
    messages.change(insert::coord(id, coord));
    messages.change(insert::tile_info(
        id,
        TileInfo::new(Tile::Floor, FLOOR_DEPTH),
    ));
}

pub fn wall<M: PushMessages>(id: EntityId, coord: Coord, messages: &mut M) {
    messages.change(insert::coord(id, coord));
    messages.change(insert::tile_info(id, TileInfo::new(Tile::Wall, WALL_DEPTH)));
    messages.change(insert::solid(id));
    messages.change(insert::opacity(id, 1));
}

pub fn cavern_wall<M: PushMessages>(id: EntityId, coord: Coord, messages: &mut M) {
    messages.change(insert::coord(id, coord));
    messages.change(insert::tile_info(
        id,
        TileInfo::new(Tile::CavernWall, WALL_DEPTH),
    ));
    messages.change(insert::solid(id));
    messages.change(insert::opacity(id, 1));
}

pub fn door<M: PushMessages>(id: EntityId, coord: Coord, messages: &mut M) {
    messages.change(insert::coord(id, coord));
    messages.change(insert::tile_info(id, TileInfo::new(Tile::Door, WALL_DEPTH)));
    messages.change(insert::solid(id));
    messages.change(insert::door(id));
    messages.change(insert::opacity(id, 1));
}

pub fn punch<M: PushMessages>(
    id: EntityId,
    coord: Coord,
    direction: CardinalDirection,
    messages: &mut M,
) {
    messages.change(insert::punch(id));
    messages.change(insert::coord(id, coord));
    messages.change(insert::tile_info(
        id,
        TileInfo::new(Tile::Punch(direction), ANIMATION_DEPTH),
    ));
}

pub fn larvae<M: PushMessages, R: Rng>(id: EntityId, coord: Coord, messages: &mut M, rng: &mut R) {
    messages.change(insert::coord(id, coord));
    messages.change(insert::npc(
        id,
        NpcInfo {
            boss: false,
            mobile: true,
            active: false,
            fast: false,
        },
    ));
    let health = Meter::full(2);
    messages.change(insert::tile_info(
        id,
        TileInfo::with_health(Tile::Larvae, NPC_DEPTH, health),
    ));
    messages.change(insert::health_meter(id, health));
    messages.change(insert::countdown(id, 20 + rng.gen::<i32>().abs() % 40));
    messages.change(insert::transform(id, Transform::Chrysalis));
}

pub fn beetoid<M: PushMessages>(id: EntityId, coord: Coord, messages: &mut M) {
    messages.change(insert::coord(id, coord));
    messages.change(insert::npc(
        id,
        NpcInfo {
            boss: false,
            mobile: true,
            active: false,
            fast: false,
        },
    ));
    let health = Meter::full(3);
    messages.change(insert::tile_info(
        id,
        TileInfo::with_health(Tile::Beetoid, NPC_DEPTH, health),
    ));
    messages.change(insert::health_meter(id, health));
}

pub fn aracnoid<M: PushMessages>(id: EntityId, coord: Coord, messages: &mut M) {
    messages.change(insert::coord(id, coord));
    messages.change(insert::npc(
        id,
        NpcInfo {
            boss: false,
            mobile: true,
            active: false,
            fast: true,
        },
    ));
    let health = Meter::full(2);
    messages.change(insert::tile_info(
        id,
        TileInfo::with_health(Tile::Aracnoid, NPC_DEPTH, health),
    ));
    messages.change(insert::health_meter(id, health));
}

pub fn chrysalis<M: PushMessages, R: Rng>(
    id: EntityId,
    coord: Coord,
    messages: &mut M,
    rng: &mut R,
) {
    messages.change(insert::coord(id, coord));
    messages.change(insert::npc(
        id,
        NpcInfo {
            boss: false,
            mobile: false,
            active: false,
            fast: false,
        },
    ));
    let health = Meter::full(1);
    messages.change(insert::tile_info(
        id,
        TileInfo::with_health(Tile::Chrysalis, NPC_DEPTH, health),
    ));
    messages.change(insert::health_meter(id, health));
    messages.change(insert::countdown(id, 2 + rng.gen::<i32>().abs() % 2));
    if rng.gen() {
        messages.change(insert::transform(id, Transform::Aracnoid));
    } else {
        messages.change(insert::transform(id, Transform::Beetoid));
    }
}

pub fn egg<M: PushMessages, R: Rng>(id: EntityId, coord: Coord, messages: &mut M, rng: &mut R) {
    messages.change(insert::coord(id, coord));
    messages.change(insert::npc(
        id,
        NpcInfo {
            boss: false,
            mobile: false,
            active: false,
            fast: false,
        },
    ));
    let health = Meter::full(3);
    messages.change(insert::tile_info(
        id,
        TileInfo::with_health(Tile::Egg, NPC_DEPTH, health),
    ));
    messages.change(insert::health_meter(id, health));
    messages.change(insert::countdown(id, 8 + rng.gen::<i32>().abs() % 30));
    messages.change(insert::transform(id, Transform::Larvae));
}

pub fn super_egg<M: PushMessages, R: Rng>(
    id: EntityId,
    coord: Coord,
    messages: &mut M,
    rng: &mut R,
) {
    messages.change(insert::coord(id, coord));
    messages.change(insert::npc(
        id,
        NpcInfo {
            boss: false,
            mobile: false,
            active: false,
            fast: false,
        },
    ));
    let health = Meter::full(8);
    messages.change(insert::tile_info(
        id,
        TileInfo::with_health(Tile::SuperEgg, NPC_DEPTH, health),
    ));
    messages.change(insert::health_meter(id, health));
    let max = 200 + rng.gen::<i32>().abs() % 100;
    messages.change(insert::countdown(id, max));
    messages.change(insert::countdown_max(id, max));
    messages.change(insert::transform(id, Transform::Queen));
}

pub fn queen<M: PushMessages>(id: EntityId, coord: Coord, boss: bool, messages: &mut M) {
    messages.change(insert::coord(id, coord));
    messages.change(insert::npc(
        id,
        NpcInfo {
            boss,
            active: boss,
            mobile: true,
            fast: false,
        },
    ));
    let health = Meter::full(10);
    messages.change(insert::tile_info(
        id,
        TileInfo {
            tile: Tile::Queen,
            depth: NPC_DEPTH,
            damage_flash: false,
            boss,
            health_meter: Some(health),
            countdown: None,
        },
    ));
    messages.change(insert::health_meter(id, health));
}

pub fn stairs<M: PushMessages>(id: EntityId, coord: Coord, messages: &mut M) {
    messages.change(insert::coord(id, coord));
    messages.change(insert::stairs(id));
    messages.change(insert::tile_info(
        id,
        TileInfo::new(Tile::Stairs, STAIRS_DEPTH),
    ));
}

pub fn exit<M: PushMessages>(id: EntityId, coord: Coord, messages: &mut M) {
    messages.change(insert::coord(id, coord));
    messages.change(insert::exit(id));
    messages.change(insert::tile_info(
        id,
        TileInfo::new(Tile::Exit, STAIRS_DEPTH),
    ));
}

pub fn bullet<M: PushMessages>(
    id: EntityId,
    coord: Coord,
    direction: CardinalDirection,
    range: u32,
    messages: &mut M,
) {
    messages.change(insert::slide_direction(id, direction));
    messages.change(insert::bullet(id, range));
    messages.change(insert::coord(id, coord));
    messages.change(insert::tile_info(
        id,
        TileInfo::new(Tile::Bullet, BULLET_DEPTH),
    ));
}

pub fn rail_gun_shot<M: PushMessages>(
    id: EntityId,
    coord: Coord,
    direction: CardinalDirection,
    messages: &mut M,
) {
    messages.change(insert::rail_gun_shot(id));
    messages.change(insert::coord(id, coord));
    let tile = match direction {
        CardinalDirection::North | CardinalDirection::South => Tile::RailGunShotVertical,
        CardinalDirection::East | CardinalDirection::West => Tile::RailGunShotHorizontal,
    };
    messages.change(insert::tile_info(
        id,
        TileInfo::new(tile, RAIL_GUN_SHOT_DEPTH),
    ));
}

pub fn health_pickup<M: PushMessages>(id: EntityId, coord: Coord, messages: &mut M) {
    messages.change(insert::coord(id, coord));
    messages.change(insert::pickup(id, Pickup::Health));
    messages.change(insert::tile_info(
        id,
        TileInfo::new(Tile::HealthPickup, PICKUP_DEPTH),
    ));
}

pub fn ammo_pickup<M: PushMessages>(id: EntityId, coord: Coord, messages: &mut M) {
    messages.change(insert::coord(id, coord));
    messages.change(insert::pickup(id, Pickup::Ammo));
    messages.change(insert::tile_info(
        id,
        TileInfo::new(Tile::AmmoPickup, PICKUP_DEPTH),
    ));
}

pub fn rail_gun_ammo_pickup<M: PushMessages>(id: EntityId, coord: Coord, messages: &mut M) {
    messages.change(insert::coord(id, coord));
    messages.change(insert::pickup(id, Pickup::RailGunAmmo));
    messages.change(insert::tile_info(
        id,
        TileInfo::new(Tile::RailGunAmmoPickup, PICKUP_DEPTH),
    ));
}

pub fn kevlar_pickup<M: PushMessages>(id: EntityId, coord: Coord, messages: &mut M) {
    messages.change(insert::coord(id, coord));
    messages.change(insert::pickup(id, Pickup::Kevlar));
    messages.change(insert::tile_info(
        id,
        TileInfo::new(Tile::KevlarPickup, PICKUP_DEPTH),
    ));
}
