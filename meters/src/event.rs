pub enum ExternalEvent {
    GameOver,
}

pub enum Event {
    External(ExternalEvent),
    NextLevel,
}
