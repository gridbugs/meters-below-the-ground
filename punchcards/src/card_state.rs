use std::mem;
use std::collections::VecDeque;
use rand::Rng;
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

    pub fn hand_is_full(&self) -> bool {
        self.hand.is_full()
    }

    fn check_invariant(&self) {
        assert!(self.hand.is_full());
        self.check_after();
    }

    fn check_after(&self) {
        assert!(self.queue.len() <= self.queue_size);
        if !self.deck.is_empty() {
            assert!(!self.queue.is_empty());
        }
    }

    pub fn play(&mut self, direction: CardinalDirection) -> Card {
        self.check_invariant();

        let card = if let Some(card) = self.queue.pop_front() {
            self.hand.insert(direction, card)
        } else {
            self.hand.take(direction)
        }.expect("Card is not in hand");

        if let Some(card) = self.deck.pop() {
            self.queue.push_back(card);
        }

        while !self.deck.is_empty() && self.queue.len() < self.queue_size {
            if let Some(card) = self.deck.pop() {
                self.queue.push_back(card);
            }
        }

        self.discard_deck.push(card);

        self.check_after();

        card
    }

    pub fn add_card<R: Rng>(&mut self, card: Card, rng: &mut R) {
        self.check_invariant();

        if self.queue.len() < self.queue_size {
            self.queue.push_back(card);
        } else {
            if self.deck.is_empty() {
                self.deck.push(card);
            } else {
                let index: usize = rng.gen();
                let to_push = mem::replace(&mut self.deck[index], card);
                self.deck.push(to_push);
            }
        }

        self.check_invariant();
    }

    pub fn hand(&self) -> &CardinalDirectionTable<Card> { &self.hand }
    pub fn deck(&self) -> &Vec<Card> { &self.deck }
    pub fn discard_deck(&self) -> &Vec<Card> { &self.discard_deck }
    pub fn queue(&self) -> &VecDeque<Card> { &self.queue }
    pub fn queue_size(&self) -> usize { self.queue_size }
}
