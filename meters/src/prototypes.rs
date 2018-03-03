use direction::CardinalDirection;
use entity_store::*;
use grid_2d::Coord;
use tile::Tile;
use tile_info::TileInfo;
use message_queues::PushMessages;
use meter::Meter;

const FLOOR_DEPTH: i32 = 1;
const WALL_DEPTH: i32 = 2;
const STAIRS_DEPTH: i32 = 3;
const BULLET_DEPTH: i32 = 4;
const NPC_DEPTH: i32 = 5;
const PLAYER_DEPTH: i32 = 6;
const ANIMATION_DEPTH: i32 = 7;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Prototype {
    Punch(EntityId, Coord, CardinalDirection),
}

impl Prototype {
    pub fn instantiate<M: PushMessages>(self, messages: &mut M) -> EntityId {
        match self {
            Prototype::Punch(id, coord, direction) => {
                punch(id, coord, direction, messages);
                id
            }
        }
    }
}

pub fn player<M: PushMessages>(id: EntityId, coord: Coord, messages: &mut M) {
    messages.change(insert::coord(id, coord));
    messages.change(insert::player(id));
    messages.change(insert::collider(id));
    messages.change(insert::tile_info(
        id,
        TileInfo::new(Tile::Player, PLAYER_DEPTH),
    ));
    messages.change(insert::health_meter(id, Meter::full(8)));
    messages.change(insert::gun_ammo_meter(id, Meter::full(8)));
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

pub fn larvae<M: PushMessages>(id: EntityId, coord: Coord, messages: &mut M) {
    messages.change(insert::coord(id, coord));
    messages.change(insert::npc(id));
    messages.change(insert::hit_points(id, 2));
    messages.change(insert::tile_info(
        id,
        TileInfo::new(Tile::Larvae, NPC_DEPTH),
    ));
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
    messages: &mut M,
) {
    messages.change(insert::slide_direction(id, direction));
    messages.change(insert::bullet(id));
    messages.change(insert::coord(id, coord));
    messages.change(insert::tile_info(
        id,
        TileInfo::new(Tile::Bullet, BULLET_DEPTH),
    ));
}
