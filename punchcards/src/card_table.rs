use direction::CardinalDirection;
use card::*;
use card_deck::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardSlot {
    current: Option<Card>,
    next: Option<Card>,
}

impl CardSlot {
    fn new() -> Self {
        CardSlot {
            current: None,
            next: None,
        }
    }
    pub fn current(&self) -> Option<Card> { self.current }
    pub fn next(&self) -> Option<Card> { self.next }
    fn is_empty(&self) -> bool {
        self.current.is_none() && self.next.is_none()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardTable {
    pub north: CardSlot,
    pub east: CardSlot,
    pub south: CardSlot,
    pub west: CardSlot,
}

impl CardTable {
    pub fn new(deck: &mut CardDeck) -> Self {
        let mut north = CardSlot::new();
        let mut east = CardSlot::new();
        let mut south = CardSlot::new();
        let mut west = CardSlot::new();

        north.next = deck.draw();
        east.next = deck.draw();
        south.next = deck.draw();
        west.next = deck.draw();

        north.current = deck.draw();
        east.current = deck.draw();
        south.current = deck.draw();
        west.current = deck.draw();

        CardTable {
            north,
            east,
            south,
            west,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.north.is_empty() &&
            self.east.is_empty() &&
            self.south.is_empty() &&
            self.west.is_empty()
    }

    pub fn take_replacing(&mut self, direction: CardinalDirection, deck: &mut CardDeck) -> Option<Card> {
        let slot = match direction {
            CardinalDirection::North => &mut self.north,
            CardinalDirection::East => &mut self.east,
            CardinalDirection::South => &mut self.south,
            CardinalDirection::West => &mut self.west,
        };

        let current = slot.current;
        slot.current = slot.next;
        slot.next = None;

        if slot.current.is_none() {
            slot.current = deck.draw();
        }

        slot.next = deck.draw();

        current
    }
}
