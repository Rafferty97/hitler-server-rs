use super::{board::Board, party::Party};
use rand::prelude::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::iter::repeat;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Deck {
    /// Total number of liberal cards in the deck, discard pile and game board
    liberal: usize,
    /// Total number of fascist cards in the deck, discard pile and game board
    fascist: usize,
    /// Total number of communist cards in the deck, discard pile and game board
    communist: usize,
    /// The current draw deck
    deck: Vec<Party>,
}

impl Deck {
    pub fn new(communists: bool) -> Self {
        let (liberal, fascist, communist) = match communists {
            false => (6, 11, 0),
            true => (6, 14, 8),
        };
        Self { liberal, fascist, communist, deck: vec![] }
    }

    /// Shuffles the discard pile into the deck, if there are fewer than three cards in the draw deck.
    pub fn check_shuffle(&mut self, board: &Board, rng: &mut impl Rng) {
        if self.deck.len() < 3 {
            self.shuffle(board, rng);
        }
    }

    /// Shuffles the discard pile into the deck.
    pub fn shuffle(&mut self, board: &Board, rng: &mut impl Rng) {
        let liberal = self.liberal - board.liberal_cards;
        let fascist = self.fascist - board.fascist_cards;
        let communist = self.communist - board.communist_cards;

        self.deck.clear();
        self.deck.extend(repeat(Party::Liberal).take(liberal));
        self.deck.extend(repeat(Party::Fascist).take(fascist));
        self.deck.extend(repeat(Party::Communist).take(communist));
        self.deck.shuffle(rng);
    }

    /// Shuffles two communist cards and one liberal card into the deck.
    pub fn five_year_plan(&mut self, rng: &mut impl Rng) {
        self.communist += 2;
        self.liberal += 1;
        self.deck.push(Party::Communist);
        self.deck.push(Party::Communist);
        self.deck.push(Party::Liberal);
        self.deck.shuffle(rng);
    }

    /// Draws the top card from the deck.
    pub fn draw_one(&mut self) -> Party {
        self.deck.pop().unwrap()
    }

    /// Draws the top three cards from the deck.
    pub fn draw_three(&mut self) -> [Party; 3] {
        let mut cards = [
            self.deck.pop().unwrap(),
            self.deck.pop().unwrap(),
            self.deck.pop().unwrap(),
        ];
        cards.reverse();
        cards
    }

    /// The number of cards in the draw pile.
    pub fn count(&self) -> usize {
        self.deck.len()
    }

    /// Peeks at the top three cards in the draw pile.
    pub fn peek_three(&self) -> [Party; 3] {
        self.deck[self.deck.len() - 3..].try_into().unwrap()
    }
}
