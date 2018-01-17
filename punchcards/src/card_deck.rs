use card::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardDeck {
    cards: Vec<Card>,
}

impl CardDeck {
    pub fn new<I: IntoIterator<Item=Card>>(cards: I) -> Self {
        Self {
            cards: cards.into_iter().collect(),
        }
    }
    pub fn draw(&mut self) -> Option<Card> {
        self.cards.pop()
    }
    pub fn add_to_top(&mut self, card: Card) {
        self.cards.push(card);
    }
}
