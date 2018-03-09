use std::time::Duration;
use grid_2d::Coord;
use direction::CardinalDirection;
use animation::*;
use entity_store::EntityId;
use prototypes::Prototype;
use timing;
use message_queues::PushMessages;

pub fn punch<M: PushMessages>(
    id: EntityId,
    coord: Coord,
    direction: CardinalDirection,
    messages: &mut M,
) {
    let punch = Prototype::Punch(id, coord, direction);
    temporary_at_coord(
        coord,
        punch,
        Duration::from_millis(timing::PUNCH_MILLIS),
        messages,
    );
}

pub fn metabol_wave<M: PushMessages>(
    id: EntityId,
    coord: Coord,
    leader: bool,
    left: bool,
    right: bool,
    direction: CardinalDirection,
    range: i32,
    messages: &mut M,
) {
    let ms = if leader {
        timing::WAVE_MILLIS
    } else {
        timing::WAVE_MILLIS / 2
    };
    let wave = Prototype::MetabolWave(id, coord, leader, left, right, direction, range);
    temporary_at_coord(
        coord,
        wave,
        Duration::from_millis(ms),
        messages,
    );
}

pub fn push_wave<M: PushMessages>(
    id: EntityId,
    coord: Coord,
    leader: bool,
    left: bool,
    right: bool,
    direction: CardinalDirection,
    range: i32,
    messages: &mut M,
) {
    let ms = if leader {
        timing::WAVE_MILLIS
    } else {
        timing::WAVE_MILLIS / 2
    };
    let wave = Prototype::PushWave(id, coord, leader, left, right, direction, range);
    temporary_at_coord(
        coord,
        wave,
        Duration::from_millis(ms),
        messages,
    );
}

pub fn rail_gun_shot<M: PushMessages>(
    id: EntityId,
    coord: Coord,
    direction: CardinalDirection,
    messages: &mut M,
) {
    let shot = Prototype::RailGunShot(id, coord, direction);
    temporary_at_coord(
        coord,
        shot,
        Duration::from_millis(timing::RAIL_GUN_SHOT_MILLIS),
        messages,
    );
}

pub fn bullet<M: PushMessages>(id: EntityId, messages: &mut M) {
    slide(
        id,
        Duration::from_millis(timing::BULLET_SLIDE_MILLIS),
        messages,
    );
}

pub fn damage_flash<M: PushMessages>(id: EntityId, messages: &mut M) {
    start_animation(
        AnimationState::DamageFlash(id, Duration::from_millis(timing::DAMAGE_FLASH_MILLIS)),
        None,
        messages,
    );
}

pub fn rail_gun_damage_flash<M: PushMessages>(id: EntityId, messages: &mut M) {
    start_animation(
        AnimationState::DamageFlash(
            id,
            Duration::from_millis(timing::DAMAGE_FLASH_MILLIS + timing::RAIL_GUN_SHOT_MILLIS),
        ),
        None,
        messages,
    );
}
