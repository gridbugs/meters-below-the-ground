use std::time::Duration;
use grid_2d::Coord;
use entity_store::*;
use append::Append;
use reaction::Reaction;
use prototypes::Prototype;

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
    pub fn step<A: Append<Reaction> + Append<EntityChange>>(
        self,
        period: Duration,
        reactions: &mut A,
    ) -> AnimationStatus {
        match self.state {
            AnimationState::RemoveEntity(id, remaining) => {
                if period > remaining {
                    reactions.append(Reaction::RemoveEntity(id));
                    AnimationStatus::Finished
                } else {
                    reactions.append(Reaction::StartAnimation(Animation::new(
                        self.channel,
                        AnimationState::RemoveEntity(id, remaining - period),
                    )));
                    AnimationStatus::Continuing
                }
            }
            AnimationState::TemporaryEntity(prototype, remaining) => {
                let id = prototype.instantiate(reactions);

                reactions.append(Reaction::StartAnimation(Animation::new(
                    self.channel,
                    AnimationState::RemoveEntity(id, remaining - period),
                )));

                AnimationStatus::Continuing
            }
        }
    }
}
