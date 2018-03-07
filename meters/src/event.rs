use alert::*;

#[derive(Debug)]
pub enum AscendStatus {
    CompleteGoal,
    IncompleteGoal,
    NoGoal,
}

#[derive(Debug)]
pub enum ExternalEvent {
    Lose,
    Win,
    Ascend(AscendStatus),
    Alert(Alert),
}

#[derive(Debug)]
pub enum Event {
    External(ExternalEvent),
}
