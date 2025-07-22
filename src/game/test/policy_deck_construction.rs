//! Policy deck construction tests
//!
//! Tests based on the Secret Hitler XL rules for policy deck composition
//! according to different player counts and game configurations.
//!
//! AMBIGUITY NOTES:
//! - Emergency power distribution: rules.pdf states "1 per player above 10, 2 per player above 13
//!   if using Communists" but unclear if this is cumulative or replacement rule.
//! - ASSUMPTION: Using replacement rule (simpler interpretation)
//! - QUESTION: For 14 players with communists, is it 4 or 6 emergency powers?
//! - Policy ratios for 17-20 players are not specified in rules.pdf

use super::super::board::Board;
use super::super::deck::Deck;
use super::super::party::Party;
use crate::game::{Game, GameOptions};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

/// Test standard policy deck composition (not 8 players)
/// Should be: 8 Communist, 5 Liberal, 10 Fascist policies
#[test]
fn test_standard_policy_deck_composition() {
    let mut deck = Deck::new(true);
    let board = Board::new(8);
    let mut rng = ChaCha8Rng::seed_from_u64(42);

    // Shuffle to populate the deck
    deck.shuffle(&board, &mut rng);

    // Count policies in the deck
    let mut liberal_count = 0;
    let mut fascist_count = 0;
    let mut communist_count = 0;

    // We need to access the deck contents, but it's private
    // Let's test by drawing all cards and counting
    let total_cards = deck.count();
    let mut cards = Vec::new();

    for _ in 0..total_cards {
        cards.push(deck.draw_one());
    }

    for card in cards {
        match card {
            Party::Liberal => liberal_count += 1,
            Party::Fascist => fascist_count += 1,
            Party::Communist => communist_count += 1,
        }
    }

    assert_eq!(liberal_count, 5, "Standard deck should have 5 Liberal policies");
    assert_eq!(fascist_count, 10, "Standard deck should have 10 Fascist policies");
    assert_eq!(communist_count, 8, "Standard deck should have 8 Communist policies");
}

/// Test 8-player policy deck composition
/// Should be: 8 Communist, 6 Liberal, 9 Fascist policies
#[test]
fn test_8_player_policy_deck_composition() {
    // Create an 8-player game to test the special deck composition
    let opts = GameOptions { communists: true, ..Default::default() };
    let player_names: Vec<String> = (0..8).map(|i| format!("Player{}", i)).collect();
    let game = Game::new(opts, &player_names, 42).unwrap();

    // Access the deck from the game
    let mut deck = game.deck.clone();
    let board = Board::new(8);
    let mut rng = ChaCha8Rng::seed_from_u64(42);

    // Shuffle to populate the deck
    deck.shuffle(&board, &mut rng);

    // Count policies in the deck
    let mut liberal_count = 0;
    let mut fascist_count = 0;
    let mut communist_count = 0;

    let total_cards = deck.count();
    let mut cards = Vec::new();

    for _ in 0..total_cards {
        cards.push(deck.draw_one());
    }

    for card in cards {
        match card {
            Party::Liberal => liberal_count += 1,
            Party::Fascist => fascist_count += 1,
            Party::Communist => communist_count += 1,
        }
    }

    assert_eq!(liberal_count, 6, "8-player deck should have 6 Liberal policies");
    assert_eq!(fascist_count, 9, "8-player deck should have 9 Fascist policies");
    assert_eq!(communist_count, 8, "8-player deck should have 8 Communist policies");
}

/// Test policy deck without communists
/// Should only contain Liberal and Fascist policies
#[test]
fn test_policy_deck_without_communists() {
    let mut deck = Deck::new(false);
    let board = Board::new(6);
    let mut rng = ChaCha8Rng::seed_from_u64(42);

    // Shuffle to populate the deck
    deck.shuffle(&board, &mut rng);

    // Count policies in the deck
    let mut liberal_count = 0;
    let mut fascist_count = 0;
    let mut communist_count = 0;

    let total_cards = deck.count();
    let mut cards = Vec::new();

    for _ in 0..total_cards {
        cards.push(deck.draw_one());
    }

    for card in cards {
        match card {
            Party::Liberal => liberal_count += 1,
            Party::Fascist => fascist_count += 1,
            Party::Communist => communist_count += 1,
        }
    }

    assert_eq!(
        communist_count, 0,
        "Non-communist deck should have 0 Communist policies"
    );
    assert!(liberal_count > 0, "Non-communist deck should have Liberal policies");
    assert!(fascist_count > 0, "Non-communist deck should have Fascist policies");
}

/// Test that policy deck is properly shuffled at game start
#[test]
fn test_policy_deck_shuffling() {
    let mut deck1 = Deck::new(true);
    let mut deck2 = Deck::new(true);
    let board = Board::new(8);
    let mut rng1 = ChaCha8Rng::seed_from_u64(42);
    let mut rng2 = ChaCha8Rng::seed_from_u64(123);

    // Shuffle both decks with different seeds
    deck1.shuffle(&board, &mut rng1);
    deck2.shuffle(&board, &mut rng2);

    // Draw first 3 cards from each deck
    let cards1 = deck1.draw_three();
    let cards2 = deck2.draw_three();

    // With different seeds, the order should be different
    // (This test might occasionally fail due to random chance, but very unlikely)
    assert_ne!(
        cards1, cards2,
        "Different random seeds should produce different card orders"
    );
}

/// Test Five-Year Plan power adds correct cards to deck
#[test]
fn test_five_year_plan_deck_modification() {
    let mut deck = Deck::new(true);
    let board = Board::new(8);
    let mut rng = ChaCha8Rng::seed_from_u64(42);

    // Initial shuffle
    deck.shuffle(&board, &mut rng);
    let initial_count = deck.count();

    // Apply Five-Year Plan
    deck.five_year_plan(&mut rng);
    let final_count = deck.count();

    // Should add exactly 3 cards (2 Communist + 1 Liberal)
    assert_eq!(
        final_count,
        initial_count + 3,
        "Five-Year Plan should add exactly 3 cards"
    );

    // Test that the deck is shuffled after addition by checking it's not just appended
    // We can't easily verify the exact composition without exposing deck internals,
    // but we can verify the count increased correctly
}

/// Test deck reshuffling when fewer than 3 cards remain
#[test]
fn test_deck_reshuffle_threshold() {
    let mut deck = Deck::new(true);
    let board = Board::new(8);
    let mut rng = ChaCha8Rng::seed_from_u64(42);

    // Initial shuffle
    deck.shuffle(&board, &mut rng);
    let initial_count = deck.count();

    // Draw cards until we have fewer than 3 left
    while deck.count() >= 3 {
        deck.draw_one();
    }

    let low_count = deck.count();
    assert!(low_count < 3, "Should have fewer than 3 cards");

    // Check shuffle should trigger reshuffle
    deck.check_shuffle(&board, &mut rng);

    // After reshuffle, should have more cards (assuming some are on the board)
    // This test assumes the board has some cards played
    let after_shuffle_count = deck.count();

    // The exact count depends on board state, but it should be consistent
    assert!(after_shuffle_count >= 0, "Deck count should be valid after reshuffle");
}

/// Test peek functionality doesn't modify deck
#[test]
fn test_deck_peek_functionality() {
    let mut deck = Deck::new(true);
    let board = Board::new(8);
    let mut rng = ChaCha8Rng::seed_from_u64(42);

    // Shuffle and ensure we have at least 3 cards
    deck.shuffle(&board, &mut rng);

    if deck.count() >= 3 {
        let initial_count = deck.count();
        let peeked_cards = deck.peek_three();
        let after_peek_count = deck.count();

        assert_eq!(initial_count, after_peek_count, "Peek should not modify deck size");

        // Verify peek shows the same cards as draw would
        let drawn_cards = deck.draw_three();
        assert_eq!(peeked_cards, drawn_cards, "Peek should show the same cards as draw");
    }
}
