use std::time::Duration;
use grid_2d::Coord;
use entity_store::*;
use prototypes::Prototype;
use message_queues::PushMessages;

pub fn start_animation<M: PushMessages>(
    initial: AnimationState,
    channel: Option<AnimationChannel>,
    messages: &mut M,
) {
    messages.animate(Animation::new(initial, channel));
}

pub fn temporary_at_coord<M: PushMessages>(
    coord: Coord,
    prototype: Prototype,
    duration: Duration,
    messages: &mut M,
) {
    start_animation(
        AnimationState::TemporaryEntity(prototype, duration),
        Some(AnimationChannel::Coord(coord)),
        messages,
    );
}

pub fn slide<M: PushMessages>(id: EntityId, period: Duration, messages: &mut M) {
    start_animation(
        AnimationState::Slide {
            id,
            remaining: period,
            reset_period: period,
        },
        None,
        messages,
    );
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimationChannel {
    Coord(Coord),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AnimationState {
    RemoveEntity(EntityId, Duration),
    TemporaryEntity(Prototype, Duration),
    Slide {
        id: EntityId,
        remaining: Duration,
        reset_period: Duration,
    },
    DamageFlash(EntityId, Duration),
    RemoveDamageFlash(EntityId, Duration),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Animation {
    pub state: AnimationState,
    pub channel: Option<AnimationChannel>,
}

impl Animation {
    pub fn new(initial_state: AnimationState, channel: Option<AnimationChannel>) -> Self {
        Self {
            state: initial_state,
            channel,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum AnimationStatus {
    ContinuingOnChannel(AnimationChannel),
    Continuing,
    Finished,
}

impl AnimationStatus {
    fn continuing(channel: Option<AnimationChannel>) -> Self {
        if let Some(channel) = channel {
            AnimationStatus::ContinuingOnChannel(channel)
        } else {
            AnimationStatus::Continuing
        }
    }
}

impl Animation {
    pub fn step<M: PushMessages>(
        self,
        period: Duration,
        entity_store: &EntityStore,
        messages: &mut M,
    ) -> AnimationStatus {
        match self.state {
            AnimationState::RemoveEntity(id, remaining) => {
                if period > remaining {
                    messages.remove(id);
                    AnimationStatus::Finished
                } else {
                    messages.animate(Animation::new(
                        AnimationState::RemoveEntity(id, remaining - period),
                        self.channel,
                    ));

                    AnimationStatus::continuing(self.channel)
                }
            }
            AnimationState::TemporaryEntity(prototype, remaining) => {
                if remaining > period {
                    let id = prototype.instantiate(messages);
                    messages.animate(Animation::new(
                            AnimationState::RemoveEntity(id, remaining - period),
                            self.channel,
                            ));
                    AnimationStatus::continuing(self.channel)
                } else {
                    AnimationStatus::Finished
                }
            }
            AnimationState::Slide {
                id,
                remaining,
                reset_period,
            } => {
                if period > remaining {
                    if let Some((coord, direction)) = entity_store
                        .slide_direction
                        .get(&id)
                        .cloned()
                        .and_then(|direction| {
                            entity_store
                                .coord
                                .get(&id)
                                .cloned()
                                .map(|coord| (coord, direction))
                        }) {
                        let next_coord = coord + direction.coord();
                        messages.change(insert::coord(id, next_coord));

                        messages.animate(Animation::new(
                            AnimationState::Slide {
                                id,
                                remaining: reset_period,
                                reset_period,
                            },
                            self.channel,
                        ));

                        AnimationStatus::continuing(self.channel)
                    } else {
                        AnimationStatus::Finished
                    }
                } else {
                    let remaining = remaining - period;

                    messages.animate(Animation::new(
                        AnimationState::Slide {
                            id,
                            remaining,
                            reset_period,
                        },
                        self.channel,
                    ));

                    AnimationStatus::continuing(self.channel)
                }
            }
            AnimationState::DamageFlash(id, remaining) => {
                if let Some(mut tile_info) = entity_store.tile_info.get(&id).cloned() {
                    tile_info.damage_flash = true;
                    messages.change(insert::tile_info(id, tile_info));

                    messages.animate(Animation::new(
                        AnimationState::RemoveDamageFlash(id, remaining),
                        self.channel,
                    ));
                    AnimationStatus::continuing(self.channel)
                } else {
                    AnimationStatus::Finished
                }
            }
            AnimationState::RemoveDamageFlash(id, remaining) => {
                if period > remaining {
                    if let Some(mut tile_info) = entity_store.tile_info.get(&id).cloned() {
                        tile_info.damage_flash = false;
                        messages.change(insert::tile_info(id, tile_info));
                    }

                    AnimationStatus::Finished
                } else {
                    let remaining = remaining - period;

                    messages.animate(Animation::new(
                        AnimationState::DamageFlash(id, remaining),
                        self.channel,
                    ));

                    AnimationStatus::continuing(self.channel)
                }
            }
        }
    }
}
