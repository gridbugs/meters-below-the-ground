use direction::CardinalDirection;
use entity_store::*;
use cgmath::Vector2;
use append::Append;
use tile::Tile;
use tile_info::TileInfo;
use card::Card;

const ANIMATION_DEPTH: i32 = 5;
const PLAYER_DEPTH: i32 = 4;
const CARD_DEPTH: i32 = 3;
const FLOOR_DEPTH: i32 = 1;
const WALL_DEPTH: i32 = 2;

pub fn card<A: Append<EntityChange>>(id: EntityId, coord: Vector2<i32>, card: Card, tile: Tile, changes: &mut A) {
    changes.append(insert::coord(id, coord));
    changes.append(insert::tile_info(id, TileInfo::new(tile, CARD_DEPTH)));
    changes.append(insert::card(id, card));
}

pub fn player<A: Append<EntityChange>>(id: EntityId, coord: Vector2<i32>, changes: &mut A) {
    changes.append(insert::coord(id, coord));
    changes.append(insert::player(id));
    changes.append(insert::collider(id));
    changes.append(insert::tile_info(id, TileInfo::new(Tile::Player, PLAYER_DEPTH)));
}

pub fn floor<A: Append<EntityChange>>(id: EntityId, coord: Vector2<i32>, changes: &mut A) {
    changes.append(insert::coord(id, coord));
    changes.append(insert::tile_info(id, TileInfo::new(Tile::Floor, FLOOR_DEPTH)));
}

pub fn wall<A: Append<EntityChange>>(id: EntityId, coord: Vector2<i32>, changes: &mut A) {
    changes.append(insert::coord(id, coord));
    changes.append(insert::tile_info(id, TileInfo::new(Tile::Wall, WALL_DEPTH)));
    changes.append(insert::solid(id));
}

pub fn punch<A: Append<EntityChange>>(id: EntityId, coord: Vector2<i32>, direction: CardinalDirection, changes: &mut A) {
    changes.append(insert::coord(id, coord));
    changes.append(insert::tile_info(id, TileInfo::new(Tile::Punch(direction), ANIMATION_DEPTH)));
}
