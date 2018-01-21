use std::collections::VecDeque;
use direction::*;
use cardinal_direction_table::*;
use card::*;

const DEFAULT_QUEUE_SIZE: usize = 4;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardState {
    hand: CardinalDirectionTable<Card>,
    deck: Vec<Card>,
    discard_deck: Vec<Card>,
    queue: VecDeque<Card>,
    queue_size: usize,
}

impl CardState {
    pub fn new<I: IntoIterator<Item=Card>>(cards: I) -> Self {
        let discard_deck = Vec::new();
        let mut deck: Vec<_> = cards.into_iter().collect();
        let mut hand = CardinalDirectionTable::new();
        let mut queue = VecDeque::new();
        let queue_size = DEFAULT_QUEUE_SIZE;

        for direction in CardinalDirections {
            if let Some(card) = deck.pop() {
                hand.insert(direction, card);
            }
        }

        for _ in 0..queue_size {
            if let Some(card) = deck.pop() {
                queue.push_back(card);
            }
        }

        Self {
            hand,
            deck,
            discard_deck,
            queue,
            queue_size,
        }
    }

    pub fn play(&mut self, direction: CardinalDirection) -> Option<Card> {

        let card = self.hand.take(direction);

        for direction in CardinalDirections {
            if !self.hand.contains(direction) {
                if let Some(card) = self.queue.pop_front() {
                    self.hand.insert(direction, card);
                }
            }
        }

        while !self.deck.is_empty() && self.queue.len() < self.queue_size {
            if let Some(card) = self.deck.pop() {
                self.queue.push_back(card);
            }
        }

        if let Some(card) = card {
            self.discard_deck.push(card);
        }

        card
    }

    pub fn add_top(&mut self, card: Card) {
        self.deck.push(card);
    }

    pub fn hand(&self) -> &CardinalDirectionTable<Card> { &self.hand }
    pub fn deck(&self) -> &Vec<Card> { &self.deck }
    pub fn discard_deck(&self) -> &Vec<Card> { &self.discard_deck }
    pub fn queue(&self) -> &VecDeque<Card> { &self.queue }
    pub fn queue_size(&self) -> usize { self.queue_size }
}
