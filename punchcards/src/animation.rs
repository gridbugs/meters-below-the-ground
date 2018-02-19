use std::time::Duration;
use entity_store::*;
use append::Append;
use reaction::Reaction;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Animation {
    RemoveEntity(EntityId, Duration),
    Delay(Box<Animation>, Duration),
}

impl Animation {
    pub fn step<A: Append<Reaction>>(self, period: Duration, reactions: &mut A) {
        match self {
            Animation::RemoveEntity(id, remaining) => {
                if period > remaining {
                    reactions.append(Reaction::RemoveEntity(id));
                } else {
                    reactions.append(Reaction::StartAnimation(Animation::RemoveEntity(
                        id,
                        remaining - period,
                    )));
                }
            }
            Animation::Delay(next, remaining) => {
                if period > remaining {
                    reactions.append(Reaction::StartAnimation(*next));
                } else {
                    reactions.append(Reaction::StartAnimation(Animation::Delay(next, remaining - period)));
                }
            }
        }
    }
}
