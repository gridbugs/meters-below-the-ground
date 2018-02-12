use direction::CardinalDirection;
use card_state::HandIndex;

pub enum Input {
    Direction(CardinalDirection),
    SelectCard(HandIndex),
    Wait,
}
