use direction::CardinalDirection;
use entity_store::*;
use grid_2d::Coord;
use append::Append;
use tile::Tile;
use tile_info::TileInfo;
use card::Card;

const ANIMATION_DEPTH: i32 = 6;
const PLAYER_DEPTH: i32 = 5;
const NPC_DEPTH: i32 = 4;
const CARD_DEPTH: i32 = 3;
const FLOOR_DEPTH: i32 = 1;
const WALL_DEPTH: i32 = 2;

pub fn card<A: Append<EntityChange>>(
    id: EntityId,
    coord: Coord,
    card: Card,
    tile: Tile,
    changes: &mut A,
) {
    changes.append(insert::coord(id, coord));
    changes.append(insert::tile_info(id, TileInfo::new(tile, CARD_DEPTH)));
    changes.append(insert::card(id, card));
}

pub fn player<A: Append<EntityChange>>(id: EntityId, coord: Coord, changes: &mut A) {
    changes.append(insert::coord(id, coord));
    changes.append(insert::player(id));
    changes.append(insert::collider(id));
    changes.append(insert::tile_info(
        id,
        TileInfo::new(Tile::Player, PLAYER_DEPTH),
    ));
}

pub fn floor<A: Append<EntityChange>>(id: EntityId, coord: Coord, changes: &mut A) {
    changes.append(insert::coord(id, coord));
    changes.append(insert::tile_info(
        id,
        TileInfo::new(Tile::Floor, FLOOR_DEPTH),
    ));
}

pub fn wall<A: Append<EntityChange>>(id: EntityId, coord: Coord, changes: &mut A) {
    changes.append(insert::coord(id, coord));
    changes.append(insert::tile_info(id, TileInfo::new(Tile::Wall, WALL_DEPTH)));
    changes.append(insert::solid(id));
}

pub fn punch<A: Append<EntityChange>>(
    id: EntityId,
    coord: Coord,
    direction: CardinalDirection,
    changes: &mut A,
) {
    changes.append(insert::punch(id));
    changes.append(insert::coord(id, coord));
    changes.append(insert::tile_info(
        id,
        TileInfo::new(Tile::Punch(direction), ANIMATION_DEPTH),
    ));
}

pub fn target_dummy<A: Append<EntityChange>>(id: EntityId, coord: Coord, changes: &mut A) {
    changes.append(insert::coord(id, coord));
    changes.append(insert::npc(id));
    changes.append(insert::hit_points(id, 2));
    changes.append(insert::tile_info(
        id,
        TileInfo::new(Tile::TargetDummy, NPC_DEPTH),
    ));
}
