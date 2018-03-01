use direction::CardinalDirection;
use entity_store::*;
use grid_2d::Coord;
use tile::Tile;
use tile_info::TileInfo;
use card::Card;
use message_queues::PushMessages;

const ANIMATION_DEPTH: i32 = 6;
const PLAYER_DEPTH: i32 = 5;
const NPC_DEPTH: i32 = 4;
const CARD_DEPTH: i32 = 3;
const FLOOR_DEPTH: i32 = 1;
const WALL_DEPTH: i32 = 2;

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

pub fn card<M: PushMessages>(id: EntityId, coord: Coord, card: Card, tile: Tile, messages: &mut M) {
    messages.change(insert::coord(id, coord));
    messages.change(insert::tile_info(id, TileInfo::new(tile, CARD_DEPTH)));
    messages.change(insert::card(id, card));
}

pub fn player<M: PushMessages>(id: EntityId, coord: Coord, messages: &mut M) {
    messages.change(insert::coord(id, coord));
    messages.change(insert::player(id));
    messages.change(insert::collider(id));
    messages.change(insert::tile_info(
        id,
        TileInfo::new(Tile::Player, PLAYER_DEPTH),
    ));
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

pub fn target_dummy<M: PushMessages>(id: EntityId, coord: Coord, messages: &mut M) {
    messages.change(insert::coord(id, coord));
    messages.change(insert::npc(id));
    messages.change(insert::hit_points(id, 2));
    messages.change(insert::tile_info(
        id,
        TileInfo::new(Tile::TargetDummy, NPC_DEPTH),
    ));
}

pub fn small_robot<M: PushMessages>(id: EntityId, coord: Coord, messages: &mut M) {
    messages.change(insert::coord(id, coord));
    messages.change(insert::npc(id));
    messages.change(insert::hit_points(id, 2));
    messages.change(insert::tile_info(
        id,
        TileInfo::new(Tile::SmallRobot, NPC_DEPTH),
    ));
}
