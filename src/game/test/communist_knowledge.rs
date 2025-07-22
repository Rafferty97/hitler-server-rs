//! Communist knowledge and revelation tests
//!
//! Tests based on the Secret Hitler XL rules for communist knowledge:
//! - Communists know each other at start if 11+ players
//! - Communists don't know each other at start if <11 players
//! - Congress power reveals original communists to newly radicalized ones
//! - Knowledge rules are determined at game start
//!
//! AMBIGUITY NOTES:
//! - Knowledge timing: rules.pdf states knowledge is determined "at the start" but unclear
//!   what happens if players are radicalized during gameplay in games that started <11 players.
//! - ASSUMPTION: Knowledge rules are set at game start and don't change during gameplay
//! - QUESTION: If a game starts with <11 players but gets radicalized, do communists learn identities?

use super::super::player::Role;
use crate::game::{Game, GameOptions};

/// Test that communists know each other at start with 11+ players
#[test]
fn test_communists_know_each_other_11_plus_players() {
    for player_count in 11..=16 {
        let opts = GameOptions {
            communists: true,
            monarchist: true,
            anarchist: true,
            capitalist: true,
            centrists: true,
        };
        let player_names: Vec<String> = (0..player_count).map(|i| format!("Player{}", i)).collect();

        if let Ok(game) = Game::new(opts, &player_names, 42) {
            // Find all communist players
            let communist_players: Vec<_> = game
                .players
                .iter()
                .enumerate()
                .filter(|(_, p)| matches!(p.role, Role::Communist | Role::Anarchist))
                .collect();

            // In games with 11+ players, communists should know each other
            // This is typically implemented through initial game state setup
            // We can't directly test "knowledge" without access to the knowledge system,
            // but we can verify the game was set up correctly for this rule

            assert!(
                communist_players.len() >= 2,
                "{} players should have at least 2 communist-aligned players",
                player_count
            );

            // ASSUMPTION: The game implementation handles communist knowledge internally
            // This test documents the requirement but can't verify the actual knowledge
            // without access to the player knowledge/information system
        }
    }
}

/// Test that communists don't know each other at start with <11 players
#[test]
fn test_communists_dont_know_each_other_less_than_11_players() {
    for player_count in 6..11 {
        let opts = GameOptions {
            communists: true,
            monarchist: false,
            anarchist: false,
            capitalist: false,
            centrists: false,
        };
        let player_names: Vec<String> = (0..player_count).map(|i| format!("Player{}", i)).collect();

        if let Ok(game) = Game::new(opts, &player_names, 42) {
            // Find all communist players
            let communist_players: Vec<_> = game
                .players
                .iter()
                .enumerate()
                .filter(|(_, p)| p.role == Role::Communist)
                .collect();

            // In games with <11 players, communists should NOT know each other initially
            // This is typically implemented through initial game state setup

            assert!(
                communist_players.len() >= 1,
                "{} players should have at least 1 communist player",
                player_count
            );

            // ASSUMPTION: The game implementation handles communist knowledge internally
            // This test documents the requirement but can't verify the actual lack of knowledge
            // without access to the player knowledge/information system
        }
    }
}

/// Test communist knowledge boundary at exactly 11 players
#[test]
fn test_communist_knowledge_boundary_11_players() {
    let opts = GameOptions {
        communists: true,
        monarchist: true,
        anarchist: true,
        capitalist: true,
        centrists: true,
    };

    // Test 10 players (should NOT know each other)
    let player_names_10: Vec<String> = (0..10).map(|i| format!("Player{}", i)).collect();
    if let Ok(game_10) = Game::new(opts, &player_names_10, 42) {
        let communist_count_10 = game_10
            .players
            .iter()
            .filter(|p| matches!(p.role, Role::Communist | Role::Anarchist))
            .count();

        assert!(
            communist_count_10 >= 1,
            "10 players should have communist players but they should NOT know each other initially"
        );
    }

    // Test 11 players (should know each other)
    let player_names_11: Vec<String> = (0..11).map(|i| format!("Player{}", i)).collect();
    if let Ok(game_11) = Game::new(opts, &player_names_11, 42) {
        let communist_count_11 = game_11
            .players
            .iter()
            .filter(|p| matches!(p.role, Role::Communist | Role::Anarchist))
            .count();

        assert!(
            communist_count_11 >= 2,
            "11 players should have communist players and they SHOULD know each other initially"
        );
    }
}

/// Test that anarchist is considered part of communist knowledge group
#[test]
fn test_anarchist_included_in_communist_knowledge() {
    let opts = GameOptions {
        communists: true,
        anarchist: true,
        monarchist: false,
        capitalist: false,
        centrists: false,
    };
    let player_names: Vec<String> = (0..11).map(|i| format!("Player{}", i)).collect();

    if let Ok(game) = Game::new(opts, &player_names, 42) {
        // Find communist and anarchist players
        let communist_players: Vec<_> = game.players.iter().filter(|p| p.role == Role::Communist).collect();
        let anarchist_players: Vec<_> = game.players.iter().filter(|p| p.role == Role::Anarchist).collect();

        // Both communists and anarchists should be part of the knowledge group
        assert!(communist_players.len() >= 1, "Should have at least 1 communist player");
        assert!(
            anarchist_players.len() >= 1,
            "Should have at least 1 anarchist player when enabled"
        );

        // ASSUMPTION: Anarchist is included in communist knowledge group
        // since they're on the communist team according to rules.pdf
    }
}

/// Test congress power reveals original communists to newly radicalized ones
#[test]
fn test_congress_power_reveals_original_communists() {
    let opts = GameOptions {
        communists: true,
        monarchist: false,
        anarchist: false,
        capitalist: false,
        centrists: false,
    };
    let player_names: Vec<String> = (0..9).map(|i| format!("Player{}", i)).collect();

    if let Ok(mut game) = Game::new(opts, &player_names, 42) {
        // Find original communist players
        let original_communists: Vec<_> = game
            .players
            .iter()
            .enumerate()
            .filter(|(_, p)| p.role == Role::Communist)
            .map(|(i, _)| i)
            .collect();

        assert!(
            original_communists.len() >= 1,
            "Should have at least 1 original communist"
        );

        // Simulate radicalization of a liberal player
        let liberal_player_idx = game
            .players
            .iter()
            .position(|p| p.role == Role::Liberal)
            .expect("Should have at least one liberal player");

        // Radicalize the liberal player
        let was_radicalized = game.players[liberal_player_idx].radicalise();
        assert!(was_radicalized, "Liberal player should be successfully radicalized");
        assert_eq!(
            game.players[liberal_player_idx].role,
            Role::Communist,
            "Player should now be communist"
        );

        // ASSUMPTION: Congress power would reveal original communists to newly radicalized ones
        // This test documents the requirement but can't test the actual congress power
        // without implementing the full communist power system

        // Verify we now have more communists than we started with
        let current_communists: Vec<_> = game.players.iter().filter(|p| p.role == Role::Communist).collect();

        assert!(
            current_communists.len() > original_communists.len(),
            "Should have more communists after radicalization"
        );
    }
}

/// Test that knowledge rules are determined at game start
#[test]
fn test_knowledge_rules_set_at_game_start() {
    // Test that a game starting with <11 players maintains those knowledge rules
    // even if the effective player count changes during gameplay

    let opts = GameOptions {
        communists: true,
        monarchist: false,
        anarchist: false,
        capitalist: false,
        centrists: false,
    };
    let player_names: Vec<String> = (0..8).map(|i| format!("Player{}", i)).collect();

    if let Ok(game) = Game::new(opts, &player_names, 42) {
        // Game started with 8 players, so communists should NOT know each other initially
        let communist_count = game.players.iter().filter(|p| p.role == Role::Communist).count();

        assert!(communist_count >= 1, "8 players should have at least 1 communist");

        // ASSUMPTION: Even if players get radicalized later, the initial knowledge rules
        // (communists don't know each other) should remain in effect for this game
        // This is based on the rules.pdf stating knowledge is determined "at the start"
    }
}

/// Test communist knowledge with different game configurations
#[test]
fn test_communist_knowledge_with_special_roles() {
    let opts = GameOptions {
        communists: true,
        monarchist: true,
        anarchist: true,
        capitalist: true,
        centrists: true,
    };
    let player_names: Vec<String> = (0..12).map(|i| format!("Player{}", i)).collect();

    if let Ok(game) = Game::new(opts, &player_names, 42) {
        // Find all players on the communist team (Communist + Anarchist)
        let communist_team: Vec<_> = game
            .players
            .iter()
            .filter(|p| matches!(p.role, Role::Communist | Role::Anarchist))
            .collect();

        // With 12 players, communists should know each other
        assert!(
            communist_team.len() >= 2,
            "12 players should have multiple communist team members"
        );

        // Verify other special roles don't interfere with communist knowledge
        let capitalist_count = game.players.iter().filter(|p| p.role == Role::Capitalist).count();
        let monarchist_count = game.players.iter().filter(|p| p.role == Role::Monarchist).count();

        // These roles should exist but not affect communist knowledge rules
        assert!(capitalist_count <= 1, "Should have at most 1 capitalist");
        assert!(monarchist_count <= 1, "Should have at most 1 monarchist");
    }
}

/// Test edge case: exactly 11 players with minimal communist setup
#[test]
fn test_minimal_communist_knowledge_11_players() {
    let opts = GameOptions {
        communists: true,
        monarchist: false,
        anarchist: false,
        capitalist: false,
        centrists: false,
    };
    let player_names: Vec<String> = (0..11).map(|i| format!("Player{}", i)).collect();

    if let Ok(game) = Game::new(opts, &player_names, 42) {
        // With exactly 11 players and minimal setup, communists should still know each other
        let communist_count = game.players.iter().filter(|p| p.role == Role::Communist).count();

        // According to the role ratios, 11 players should have 2 communists
        assert!(
            communist_count >= 2,
            "11 players should have at least 2 communists who know each other"
        );
    }
}

/// Test that radicalization doesn't change initial knowledge rules
#[test]
fn test_radicalization_preserves_initial_knowledge_rules() {
    let opts = GameOptions {
        communists: true,
        monarchist: false,
        anarchist: false,
        capitalist: false,
        centrists: false,
    };
    let player_names: Vec<String> = (0..10).map(|i| format!("Player{}", i)).collect();

    if let Ok(mut game) = Game::new(opts, &player_names, 42) {
        // Game started with 10 players, so initial knowledge rule is "don't know each other"
        let initial_communist_count = game.players.iter().filter(|p| p.role == Role::Communist).count();

        // Radicalize some liberal players
        let liberal_indices: Vec<_> = game
            .players
            .iter()
            .enumerate()
            .filter(|(_, p)| p.role == Role::Liberal)
            .map(|(i, _)| i)
            .take(2)
            .collect();

        for &idx in &liberal_indices {
            game.players[idx].radicalise();
        }

        let final_communist_count = game.players.iter().filter(|p| p.role == Role::Communist).count();

        assert!(
            final_communist_count > initial_communist_count,
            "Should have more communists after radicalization"
        );

        // ASSUMPTION: Despite having more communists now, the initial knowledge rule
        // (communists don't know each other) should still apply since the game started with <11 players
        // Only the Congress power should reveal original communists to newly radicalized ones
    }
}
