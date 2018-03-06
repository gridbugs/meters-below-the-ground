use rand::Rng;
use entity_store::*;
use grid_2d::Coord;
use meter::*;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum GoalType {
    Escape,
    KillEggs,
    KillBoss,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GoalState {
    Escape(Coord),
    KillEggs(Vec<EntityId>),
    KillBoss(EntityId),
}

const GOAL_TYPE_CHOICES: &[GoalType] = &[
    //    GoalType::KillEggs,
    GoalType::KillBoss,
];

pub fn choose_goal_type<R: Rng>(rng: &mut R) -> GoalType {
    let index = rng.gen::<usize>() % GOAL_TYPE_CHOICES.len();
    GOAL_TYPE_CHOICES[index]
}

impl GoalState {
    pub fn typ(&self) -> GoalType {
        match self {
            &GoalState::Escape(_) => GoalType::Escape,
            &GoalState::KillEggs(_) => GoalType::KillEggs,
            &GoalState::KillBoss(_) => GoalType::KillBoss,
        }
    }
    pub fn with_goal_meters<F>(&self, entity_store: &EntityStore, mut f: F)
    where
        F: FnMut(GoalMeterInfo),
    {
        match self {
            &GoalState::Escape(_) => {}
            &GoalState::KillEggs(_) => {}
            &GoalState::KillBoss(id) => {
                if let Some(health) = entity_store.health_meter.get(&id).cloned() {
                    f(GoalMeterInfo {
                        typ: GoalMeterType::BossHealth,
                        meter: health,
                    })
                }
            }
        }
    }
    pub fn is_complete(&self, entity_store: &EntityStore) -> bool {
        match self {
            &GoalState::Escape(_) => false,
            &GoalState::KillEggs(_) => false,
            &GoalState::KillBoss(id) => !entity_store.health_meter.contains_key(&id),
        }
    }
}

#[derive(Clone, Debug, Copy)]
pub enum GoalMeterType {
    BossHealth,
}

#[derive(Clone, Debug)]
pub struct GoalMeterInfo {
    pub typ: GoalMeterType,
    pub meter: Meter,
}
