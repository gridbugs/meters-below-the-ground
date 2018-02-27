use std::time::Duration;
use append::Append;
use grid_2d::Coord;
use direction::CardinalDirection;
use animation::*;
use entity_store::EntityId;
use prototypes::Prototype;
use timing;

pub fn punch<A: Append<Animation>>(
    id: EntityId,
    coord: Coord,
    direction: CardinalDirection,
    animations: &mut A,
) {
    let punch = Prototype::Punch(id, coord, direction);
    temporary_at_coord(
        coord,
        punch,
        Duration::from_millis(timing::PUNCH_MILLIS),
        animations,
    );
}
