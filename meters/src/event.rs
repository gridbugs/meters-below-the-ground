use alert::*;

pub enum AscendStatus {
    CompleteGoal,
    IncompleteGoal,
    NoGoal,
}

pub enum ExternalEvent {
    Lose,
    Win,
    Ascend(AscendStatus),
    Alert(Alert),
}

pub enum Event {
    External(ExternalEvent),
}
