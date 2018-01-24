use std::mem;
use std::collections::{vec_deque, VecDeque};
use rand::Rng;
use direction::*;
use cardinal_direction_table::*;
use card::*;

const DEFAULT_QUEUE_SIZE: usize = 4;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deck {
    deck: Vec<Card>,
}

impl Deck {
    fn from_cards<I: IntoIterator<Item=Card>>(cards: I) -> Self {
        Self {
            deck: cards.into_iter().collect(),
        }
    }

    fn new() -> Self {
        Self {
            deck: Vec::new(),
        }
    }

    fn is_empty(&self) -> bool { self.deck.is_empty() }

    fn add_random<R: Rng>(&mut self, card: Card, rng: &mut R) {
        if self.deck.is_empty() {
            self.deck.push(card);
        } else {
            let index: usize = rng.gen();
            let to_push = mem::replace(&mut self.deck[index], card);
            self.deck.push(to_push);
        }
    }

    fn add_top(&mut self, card: Card) {
        self.deck.push(card);
    }

    fn draw(&mut self) -> Option<Card> {
        self.deck.pop()
    }

    pub fn num_cards(&self) -> usize {
        self.deck.len()
    }

    fn shuffle<R: Rng>(&mut self, rng: &mut R) {
        let size = self.deck.len();
        for index in 0..size {
            let unsorted_size = size - index;
            let offset = rng.gen::<usize>() % unsorted_size;
            let swap_index = index + offset;
            self.deck.swap(index, swap_index);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Queue {
    queue: VecDeque<Card>,
    max_size: usize,
}

impl Queue {
    fn new(max_size: usize) -> Self {
        Self {
            queue: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    fn add(&mut self, card: Card) {
        if self.queue.len() == self.max_size {
            panic!("Too many cards enqueued");
        }
        self.queue.push_back(card);
    }

    pub fn max_size(&self) -> usize { self.max_size }

    fn is_empty(&self) -> bool { self.queue.is_empty() }

    fn is_full(&self) -> bool { self.queue.len() == self.max_size }

    fn draw(&mut self) -> Option<Card> {
        self.queue.pop_front()
    }

    pub fn iter(&self) -> QueueIter {
        QueueIter {
            iter: self.queue.iter(),
            count: 0,
            max_size: self.max_size,
        }
    }
}

pub struct QueueIter<'a> {
    iter: vec_deque::Iter<'a, Card>,
    count: usize,
    max_size: usize,
}

impl<'a> Iterator for QueueIter<'a> {
    type Item = Option<&'a Card>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.count == self.max_size {
            None
        } else {
            self.count += 1;
            Some(self.iter.next())
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardState {
    hand: CardinalDirectionTable<Card>,
    deck: Deck,
    discard_deck: Deck,
    queue: Queue,
}

impl CardState {
    pub fn new<I: IntoIterator<Item=Card>, R: Rng>(cards: I, rng: &mut R) -> Self {
        let discard_deck = Deck::new();
        let mut deck = Deck::from_cards(cards);
        deck.shuffle(rng);
        let mut hand = CardinalDirectionTable::new();
        let mut queue = Queue::new(DEFAULT_QUEUE_SIZE);

        for direction in CardinalDirections {
            if let Some(card) = deck.draw() {
                hand.insert(direction, card);
            }
        }

        for _ in 0..queue.max_size() {
            if let Some(card) = deck.draw() {
                queue.add(card);
            }
        }

        Self {
            hand,
            deck,
            discard_deck,
            queue,
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
        if !self.deck.is_empty() {
            assert!(!self.queue.is_empty());
        }
    }

    pub fn play(&mut self, direction: CardinalDirection) -> Card {
        self.check_invariant();

        let card = if let Some(card) = self.queue.draw() {
            self.hand.insert(direction, card)
        } else {
            self.hand.take(direction)
        }.expect("Card is not in hand");

        if let Some(card) = self.deck.draw() {
            self.queue.add(card);
        }

        while !self.deck.is_empty() && !self.queue.is_full() {
            if let Some(card) = self.deck.draw() {
                self.queue.add(card);
            }
        }

        self.discard_deck.add_top(card);

        self.check_after();

        card
    }

    pub fn add_card<R: Rng>(&mut self, card: Card, rng: &mut R) {
        self.check_invariant();

        if !self.queue.is_full() {
            self.queue.add(card);
        } else {
            self.deck.add_random(card, rng);
        }

        self.check_invariant();
    }

    pub fn hand(&self) -> &CardinalDirectionTable<Card> { &self.hand }
    pub fn deck(&self) -> &Deck { &self.deck }
    pub fn discard_deck(&self) -> &Deck { &self.discard_deck }
    pub fn queue(&self) -> &Queue { &self.queue }
}
