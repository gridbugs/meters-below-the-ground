use direction::CardinalDirection;

pub enum Input {
    Direction(CardinalDirection),
    MeterSelect(char),
    MeterDeselect,
    Wait,
}
