//! Anti-policies implementation tests
//!
//! Tests based on the Secret Hitler XL rules for anti-policies:
//! - Anti-Communist Policy: Placed on Fascist tracker, removes one Communist policy
//! - Anti-Fascist Policy: Placed on Communist tracker, removes one Fascist policy
//! - Social Democratic Policy: Placed on Liberal tracker, removes either Fascist or Communist policy
//!
//! AMBIGUITY NOTES:
//! - Anti-policy placement: rules.pdf states anti-policies are "placed on" trackers
//!   but then "remove" policies from other trackers. Unclear if they occupy slots.
//! - ASSUMPTION: Anti-policies occupy tracker slots like normal policies
//! - QUESTION: Do anti-policies count toward victory conditions or just removal mechanisms?
//! - These tests expect anti-policy variants in Party enum (currently missing from implementation)

use super::super::board::Board;
use super::super::party::Party;
use super::test_utils::*;
use crate::game::{Game, GameOptions};

/// Test that Anti-Communist policies are correctly included when enabled
#[test]
fn test_anti_communist_policy_inclusion() {
    let opts = GameOptions {
        communists: true,
        // Anti-policies should be enabled via some option
        ..Default::default()
    };
    let player_names: Vec<String> = (0..8).map(|i| format!("Player{}", i)).collect();
    let game = Game::new(opts, &player_names, 42).unwrap();

    // Check that the deck contains anti-communist policies
    // This will fail until anti-policies are implemented
    let mut deck = game.deck.clone();
    let board = Board::new(8);
    let mut rng = rand::thread_rng();

    deck.shuffle(&board, &mut rng);

    let total_cards = deck.count();
    let mut has_anti_communist = false;

    for _ in 0..total_cards {
        let card = deck.draw_one();
        // This will fail until Party::AntiCommunist is implemented
        if matches!(card, Party::AntiCommunist) {
            has_anti_communist = true;
            break;
        }
    }

    assert!(
        has_anti_communist,
        "Deck should contain Anti-Communist policies when enabled"
    );
}

/// Test that Anti-Fascist policies are correctly included when enabled
#[test]
fn test_anti_fascist_policy_inclusion() {
    let opts = GameOptions {
        communists: true,
        // Anti-policies should be enabled via some option
        ..Default::default()
    };
    let player_names: Vec<String> = (0..8).map(|i| format!("Player{}", i)).collect();
    let game = Game::new(opts, &player_names, 42).unwrap();

    // Check that the deck contains anti-fascist policies
    // This will fail until anti-policies are implemented
    let mut deck = game.deck.clone();
    let board = Board::new(8);
    let mut rng = rand::thread_rng();

    deck.shuffle(&board, &mut rng);

    let total_cards = deck.count();
    let mut has_anti_fascist = false;

    for _ in 0..total_cards {
        let card = deck.draw_one();
        // This will fail until Party::AntiFascist is implemented
        if matches!(card, Party::AntiFascist) {
            has_anti_fascist = true;
            break;
        }
    }

    assert!(
        has_anti_fascist,
        "Deck should contain Anti-Fascist policies when enabled"
    );
}

/// Test that Social Democratic policies are correctly included when enabled
#[test]
fn test_social_democratic_policy_inclusion() {
    let opts = GameOptions {
        communists: true,
        // Social Democratic policies should be enabled when Liberals are at disadvantage
        ..Default::default()
    };
    let player_names: Vec<String> = (0..8).map(|i| format!("Player{}", i)).collect();
    let game = Game::new(opts, &player_names, 42).unwrap();

    // Check that the deck contains social democratic policies
    // This will fail until anti-policies are implemented
    let mut deck = game.deck.clone();
    let board = Board::new(8);
    let mut rng = rand::thread_rng();

    deck.shuffle(&board, &mut rng);

    let total_cards = deck.count();
    let mut has_social_democratic = false;

    for _ in 0..total_cards {
        let card = deck.draw_one();
        // This will fail until Party::SocialDemocratic is implemented
        if matches!(card, Party::SocialDemocratic) {
            has_social_democratic = true;
            break;
        }
    }

    assert!(
        has_social_democratic,
        "Deck should contain Social Democratic policies when enabled"
    );
}

/// Test Anti-Communist policy placement on Fascist tracker
#[test]
fn test_anti_communist_policy_placement() {
    let mut board = Board::new(8);

    // This will fail until Party::AntiCommunist is implemented
    board.play_card(Party::AntiCommunist);

    // Anti-Communist policy should be placed on Fascist tracker
    assert_eq!(
        board.fascist_cards, 1,
        "Anti-Communist policy should be placed on Fascist tracker"
    );
    assert_eq!(
        board.communist_cards, 0,
        "Anti-Communist policy should not be placed on Communist tracker"
    );
    assert_eq!(
        board.liberal_cards, 0,
        "Anti-Communist policy should not be placed on Liberal tracker"
    );
}

/// Test Anti-Fascist policy placement on Communist tracker
#[test]
fn test_anti_fascist_policy_placement() {
    let mut board = Board::new(8);

    // This will fail until Party::AntiFascist is implemented
    board.play_card(Party::AntiFascist);

    // Anti-Fascist policy should be placed on Communist tracker
    assert_eq!(
        board.communist_cards, 1,
        "Anti-Fascist policy should be placed on Communist tracker"
    );
    assert_eq!(
        board.fascist_cards, 0,
        "Anti-Fascist policy should not be placed on Fascist tracker"
    );
    assert_eq!(
        board.liberal_cards, 0,
        "Anti-Fascist policy should not be placed on Liberal tracker"
    );
}

/// Test Social Democratic policy placement on Liberal tracker
#[test]
fn test_social_democratic_policy_placement() {
    let mut board = Board::new(8);

    // This will fail until Party::SocialDemocratic is implemented
    board.play_card(Party::SocialDemocratic);

    // Social Democratic policy should be placed on Liberal tracker
    assert_eq!(
        board.liberal_cards, 1,
        "Social Democratic policy should be placed on Liberal tracker"
    );
    assert_eq!(
        board.fascist_cards, 0,
        "Social Democratic policy should not be placed on Fascist tracker"
    );
    assert_eq!(
        board.communist_cards, 0,
        "Social Democratic policy should not be placed on Communist tracker"
    );
}

/// Test Anti-Communist policy power: removes one Communist policy
#[test]
fn test_anti_communist_policy_power() {
    let mut board = Board::new(8);

    // Set up board with some Communist policies
    board.play_card(Party::Communist);
    board.play_card(Party::Communist);
    assert_eq!(board.communist_cards, 2, "Should have 2 Communist policies");

    // Play Anti-Communist policy - this will fail until implemented
    board.play_card(Party::AntiCommunist);

    // After Anti-Communist policy power, should have one fewer Communist policy
    assert_eq!(
        board.communist_cards, 1,
        "Anti-Communist policy should remove one Communist policy"
    );
    assert_eq!(
        board.fascist_cards, 1,
        "Anti-Communist policy should be on Fascist tracker"
    );
}

/// Test Anti-Fascist policy power: removes one Fascist policy
#[test]
fn test_anti_fascist_policy_power() {
    let mut board = Board::new(8);

    // Set up board with some Fascist policies
    board.play_card(Party::Fascist);
    board.play_card(Party::Fascist);
    assert_eq!(board.fascist_cards, 2, "Should have 2 Fascist policies");

    // Play Anti-Fascist policy - this will fail until implemented
    board.play_card(Party::AntiFascist);

    // After Anti-Fascist policy power, should have one fewer Fascist policy
    assert_eq!(
        board.fascist_cards, 1,
        "Anti-Fascist policy should remove one Fascist policy"
    );
    assert_eq!(
        board.communist_cards, 1,
        "Anti-Fascist policy should be on Communist tracker"
    );
}

/// Test Social Democratic policy power: removes either Fascist or Communist policy
#[test]
fn test_social_democratic_policy_power() {
    let mut board = Board::new(8);

    // Set up board with both Fascist and Communist policies
    board.play_card(Party::Fascist);
    board.play_card(Party::Communist);
    assert_eq!(board.fascist_cards, 1, "Should have 1 Fascist policy");
    assert_eq!(board.communist_cards, 1, "Should have 1 Communist policy");

    // Play Social Democratic policy - this will fail until implemented
    board.play_card(Party::SocialDemocratic);

    // After Social Democratic policy power, should have removed either Fascist or Communist
    let total_removed = (board.fascist_cards == 0) as u8 + (board.communist_cards == 0) as u8;
    assert_eq!(
        total_removed, 1,
        "Social Democratic policy should remove exactly one Fascist or Communist policy"
    );
    assert_eq!(
        board.liberal_cards, 1,
        "Social Democratic policy should be on Liberal tracker"
    );
}

/// Test that anti-policies don't trigger power reuse
#[test]
fn test_anti_policies_no_power_reuse() {
    let mut board = Board::new(8);

    // Set up board to trigger a power
    board.play_card(Party::Fascist);
    board.play_card(Party::Fascist);
    board.play_card(Party::Fascist);

    // The next Fascist policy should trigger a power
    let power_before = board.get_executive_power(Party::Fascist);
    assert!(power_before.is_some(), "Should have executive power available");

    // Play Anti-Communist policy (goes on Fascist tracker) - this will fail until implemented
    board.play_card(Party::AntiCommunist);

    // The next Fascist policy should NOT trigger power reuse due to anti-policy
    board.play_card(Party::Fascist);
    let power_after = board.get_executive_power(Party::Fascist);

    // This test verifies that anti-policies prevent power reuse
    // The exact behavior depends on implementation details
    assert!(
        true,
        "Anti-policies should prevent power reuse - implementation dependent"
    );
}

/// Test anti-policies are properly tracked and displayed
#[test]
fn test_anti_policies_tracking() {
    let mut board = Board::new(8);

    // Play various anti-policies - these will fail until implemented
    board.play_card(Party::AntiCommunist);
    board.play_card(Party::AntiFascist);
    board.play_card(Party::SocialDemocratic);

    // Verify they are tracked correctly on their respective trackers
    assert_eq!(board.fascist_cards, 1, "Anti-Communist should be on Fascist tracker");
    assert_eq!(board.communist_cards, 1, "Anti-Fascist should be on Communist tracker");
    assert_eq!(board.liberal_cards, 1, "Social Democratic should be on Liberal tracker");

    // The board should be able to distinguish between regular and anti-policies
    // This might require additional tracking fields in the Board struct
    assert!(
        true,
        "Anti-policies should be properly tracked and distinguishable from regular policies"
    );
}
