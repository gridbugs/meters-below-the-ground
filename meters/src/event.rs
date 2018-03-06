pub enum AscendStatus {
    CompleteGoal,
    IncompleteGoal,
    NoGoal,
}

pub enum ExternalEvent {
    Lose,
    Win,
    Ascend(AscendStatus),
}

pub enum Event {
    External(ExternalEvent),
}
