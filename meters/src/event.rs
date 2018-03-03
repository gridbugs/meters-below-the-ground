pub enum ExternalEvent {
    Lose,
    Win,
}

pub enum Event {
    External(ExternalEvent),
    NextLevel,
}
