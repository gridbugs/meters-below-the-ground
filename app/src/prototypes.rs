use entity_store::*;
use cgmath::Vector2;
use append::Append;
use tile::Tile;

const PLAYER_DEPTH: i32 = 3;
const FLOOR_DEPTH: i32 = 1;
const WALL_DEPTH: i32 = 2;

pub fn player<A: Append<EntityChange>>(id: EntityId, coord: Vector2<i32>, changes: &mut A) {
    changes.append(insert::coord(id, coord));
    changes.append(insert::player(id));
    changes.append(insert::tile(id, Tile::Player));
    changes.append(insert::collider(id));
    changes.append(insert::depth(id, PLAYER_DEPTH));
}

pub fn floor<A: Append<EntityChange>>(id: EntityId, coord: Vector2<i32>, changes: &mut A) {
    changes.append(insert::coord(id, coord));
    changes.append(insert::tile(id, Tile::Floor));
    changes.append(insert::depth(id, FLOOR_DEPTH));
}

pub fn wall<A: Append<EntityChange>>(id: EntityId, coord: Vector2<i32>, changes: &mut A) {
    changes.append(insert::coord(id, coord));
    changes.append(insert::tile(id, Tile::Wall));
    changes.append(insert::depth(id, WALL_DEPTH));
    changes.append(insert::solid(id));
}
