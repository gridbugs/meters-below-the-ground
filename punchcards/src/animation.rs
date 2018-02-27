use std::time::Duration;
use grid_2d::Coord;
use entity_store::*;
use prototypes::Prototype;
use message_queues::PushMessages;

pub fn start_animation<M: PushMessages>(
    channel: AnimationChannel,
    initial: AnimationState,
    messages: &mut M,
) {
    messages.animate(Animation::new(channel, initial));
}

pub fn temporary_at_coord<M: PushMessages>(
    coord: Coord,
    prototype: Prototype,
    duration: Duration,
    messages: &mut M,
) {
    start_animation(
        AnimationChannel::Coord(coord),
        AnimationState::TemporaryEntity(prototype, duration),
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
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Animation {
    pub state: AnimationState,
    pub channel: AnimationChannel,
}

impl Animation {
    pub fn new(channel: AnimationChannel, initial_state: AnimationState) -> Self {
        Self {
            channel,
            state: initial_state,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum AnimationStatus {
    Continuing,
    Finished,
}

impl Animation {
    pub fn step<M: PushMessages>(
        self,
        period: Duration,
        messages: &mut M,
    ) -> AnimationStatus {
        match self.state {
            AnimationState::RemoveEntity(id, remaining) => {
                if period > remaining {
                    messages.remove(id);
                    AnimationStatus::Finished
                } else {
                    messages.animate(Animation::new(
                        self.channel,
                        AnimationState::RemoveEntity(id, remaining - period),
                    ));
                    AnimationStatus::Continuing
                }
            }
            AnimationState::TemporaryEntity(prototype, remaining) => {
                let id = prototype.instantiate(messages);

                messages.animate(Animation::new(
                    self.channel,
                    AnimationState::RemoveEntity(id, remaining - period),
                ));

                AnimationStatus::Continuing
            }
        }
    }
}
