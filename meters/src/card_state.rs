use std::mem;
use std::slice;
use rand::Rng;
use card::*;

pub type HandIndex = usize;

pub type HandIter<'a> = slice::Iter<'a, Option<Card>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hand {
    cards: Vec<Option<Card>>,
}

impl Hand {
    pub fn new(max_size: usize) -> Self {
        let mut cards = Vec::new();
        cards.resize(max_size, None);
        Self { cards }
    }

    pub fn add_card(&mut self, card: Card) {
        let first_free_index = self.cards
            .iter()
            .position(Option::is_none)
            .expect("Hand is full");
        self.cards[first_free_index] = Some(card);
    }

    pub fn remove_card(&mut self, index: HandIndex) -> Card {
        mem::replace(&mut self.cards[index], None).expect("No card in position")
    }

    pub fn get(&self, index: HandIndex) -> Option<&Card> {
        self.cards.get(index).and_then(Option::as_ref)
    }

    pub fn size(&self) -> usize {
        self.cards.len()
    }

    pub fn iter(&self) -> HandIter {
        self.cards.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.cards.iter().all(Option::is_none)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deck {
    deck: Vec<Card>,
}

impl Deck {
    fn from_cards<I: IntoIterator<Item = Card>>(cards: I) -> Self {
        Self {
            deck: cards.into_iter().collect(),
        }
    }

    fn new() -> Self {
        Self { deck: Vec::new() }
    }

    pub fn add_random<R: Rng>(&mut self, card: Card, rng: &mut R) {
        if self.deck.is_empty() {
            self.deck.push(card);
        } else {
            let index = rng.gen::<usize>() % self.deck.len();
            let to_push = mem::replace(&mut self.deck[index], card);
            self.deck.push(to_push);
        }
    }

    fn draw(&mut self) -> Option<Card> {
        self.deck.pop()
    }

    pub fn num_cards(&self) -> usize {
        self.deck.len()
    }

    fn shuffle<R: Rng>(&mut self, rng: &mut R) {
        rng.shuffle(&mut self.deck);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardState {
    pub hand: Hand,
    pub deck: Deck,
    pub discard_deck: Deck,
}

impl CardState {
    pub fn new<I: IntoIterator<Item = Card>, R: Rng>(
        cards: I,
        hand_size: usize,
        rng: &mut R,
    ) -> Self {
        let discard_deck = Deck::new();
        let mut deck = Deck::from_cards(cards);
        deck.shuffle(rng);
        let hand = Hand::new(hand_size);

        let mut state = Self {
            hand,
            deck,
            discard_deck,
        };

        state.fill_hand();

        state
    }

    pub fn fill_hand(&mut self) {
        let empty_hand_spaces = self.hand.cards.iter_mut().filter(|s| s.is_none());

        for space in empty_hand_spaces {
            if let Some(card) = self.deck.draw() {
                *space = Some(card);
            } else {
                break;
            }
        }
    }
}
