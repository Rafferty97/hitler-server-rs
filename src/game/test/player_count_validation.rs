//! Player count and role ratio validation tests
//!
//! Tests based on the Secret Hitler XL rules for player counts 6-20
//! and the specific role ratios required for each player count.
//!
//! CRITICAL AMBIGUITY:
//! test.md specifies 6-20 players, but rules.pdf only provides role ratios up to 16 players.
//! The rules.pdf mentions "up to 20 players" in the introduction but doesn't specify ratios
//! for 17-20 players. These tests assume 6-20 range but implementation may only support 6-16.
//!
//! ASSUMPTION: Role ratios for 17-20 players are extrapolated from existing patterns.
//! QUESTION: Should the system support 17-20 players? What are the official ratios?

use super::super::player::Role;
use crate::game::{Game, GameOptions};

/// Test that the system accepts valid player counts (6-20 per test.md specification)
#[test]
fn test_valid_player_counts() {
    let opts = GameOptions::default();

    for player_count in 6..=20 {
        let player_names: Vec<String> = (0..player_count).map(|i| format!("Player{}", i)).collect();
        let result = Game::new(opts, &player_names, 42);

        if player_count <= 10 {
            // Standard game should work up to 10 players
            assert!(result.is_ok(), "Standard game should accept {} players", player_count);
        } else {
            // XL features needed for 11+ players
            let xl_opts = GameOptions {
                communists: true,
                monarchist: true,
                anarchist: true,
                capitalist: true,
                centrists: true,
            };
            let xl_result = Game::new(xl_opts, &player_names, 42);
            assert!(xl_result.is_ok(), "XL game should accept {} players", player_count);
        }
    }
}

/// Test that the system rejects invalid player counts (per test.md specification)
#[test]
fn test_invalid_player_counts() {
    let opts = GameOptions::default();

    // Too few players (< 6)
    for player_count in 1..6 {
        let player_names: Vec<String> = (0..player_count).map(|i| format!("Player{}", i)).collect();
        let result = Game::new(opts, &player_names, 42);
        assert!(result.is_err(), "Should reject {} players (too few)", player_count);
    }

    // Too many players (> 20)
    for player_count in 21..25 {
        let player_names: Vec<String> = (0..player_count).map(|i| format!("Player{}", i)).collect();
        let result = Game::new(opts, &player_names, 42);
        assert!(result.is_err(), "Should reject {} players (too many)", player_count);
    }
}

/// Test specific role ratios according to the rules table
#[test]
fn test_6_player_role_ratios() {
    // 6 players: 3L, 1F+H, 1C
    let opts = GameOptions { communists: true, ..Default::default() };
    let player_names: Vec<String> = (0..6).map(|i| format!("Player{}", i)).collect();
    let game = Game::new(opts, &player_names, 42).unwrap();

    let liberal_count = game.players.iter().filter(|p| p.role == Role::Liberal).count();
    let fascist_count = game.players.iter().filter(|p| p.role == Role::Fascist).count();
    let hitler_count = game.players.iter().filter(|p| p.role == Role::Hitler).count();
    let communist_count = game.players.iter().filter(|p| p.role == Role::Communist).count();

    assert_eq!(liberal_count, 3, "6 players should have 3 Liberals");
    assert_eq!(fascist_count, 1, "6 players should have 1 Fascist");
    assert_eq!(hitler_count, 1, "6 players should have 1 Hitler");
    assert_eq!(communist_count, 1, "6 players should have 1 Communist");
}

#[test]
fn test_7_player_role_ratios() {
    // 7 players: 4L, 1F+H, 1C
    let opts = GameOptions { communists: true, ..Default::default() };
    let player_names: Vec<String> = (0..7).map(|i| format!("Player{}", i)).collect();
    let game = Game::new(opts, &player_names, 42).unwrap();

    let liberal_count = game.players.iter().filter(|p| p.role == Role::Liberal).count();
    let fascist_count = game.players.iter().filter(|p| p.role == Role::Fascist).count();
    let hitler_count = game.players.iter().filter(|p| p.role == Role::Hitler).count();
    let communist_count = game.players.iter().filter(|p| p.role == Role::Communist).count();

    assert_eq!(liberal_count, 4, "7 players should have 4 Liberals");
    assert_eq!(fascist_count, 1, "7 players should have 1 Fascist");
    assert_eq!(hitler_count, 1, "7 players should have 1 Hitler");
    assert_eq!(communist_count, 1, "7 players should have 1 Communist");
}

#[test]
fn test_8_player_role_ratios() {
    // 8 players: 4L, 2F+H, 1C
    let opts = GameOptions { communists: true, ..Default::default() };
    let player_names: Vec<String> = (0..8).map(|i| format!("Player{}", i)).collect();
    let game = Game::new(opts, &player_names, 42).unwrap();

    let liberal_count = game.players.iter().filter(|p| p.role == Role::Liberal).count();
    let fascist_count = game.players.iter().filter(|p| p.role == Role::Fascist).count();
    let hitler_count = game.players.iter().filter(|p| p.role == Role::Hitler).count();
    let communist_count = game.players.iter().filter(|p| p.role == Role::Communist).count();

    assert_eq!(liberal_count, 4, "8 players should have 4 Liberals");
    assert_eq!(fascist_count, 2, "8 players should have 2 Fascists");
    assert_eq!(hitler_count, 1, "8 players should have 1 Hitler");
    assert_eq!(communist_count, 1, "8 players should have 1 Communist");
}

#[test]
fn test_9_player_role_ratios() {
    // 9 players: 5L, 2F+H, 1C
    let opts = GameOptions { communists: true, ..Default::default() };
    let player_names: Vec<String> = (0..9).map(|i| format!("Player{}", i)).collect();
    let game = Game::new(opts, &player_names, 42).unwrap();

    let liberal_count = game.players.iter().filter(|p| p.role == Role::Liberal).count();
    let fascist_count = game.players.iter().filter(|p| p.role == Role::Fascist).count();
    let hitler_count = game.players.iter().filter(|p| p.role == Role::Hitler).count();
    let communist_count = game.players.iter().filter(|p| p.role == Role::Communist).count();

    assert_eq!(liberal_count, 5, "9 players should have 5 Liberals");
    assert_eq!(fascist_count, 2, "9 players should have 2 Fascists");
    assert_eq!(hitler_count, 1, "9 players should have 1 Hitler");
    assert_eq!(communist_count, 1, "9 players should have 1 Communist");
}

#[test]
fn test_10_player_role_ratios() {
    // 10 players: 5L, 3F+H, 1C
    let opts = GameOptions { communists: true, ..Default::default() };
    let player_names: Vec<String> = (0..10).map(|i| format!("Player{}", i)).collect();
    let game = Game::new(opts, &player_names, 42).unwrap();

    let liberal_count = game.players.iter().filter(|p| p.role == Role::Liberal).count();
    let fascist_count = game.players.iter().filter(|p| p.role == Role::Fascist).count();
    let hitler_count = game.players.iter().filter(|p| p.role == Role::Hitler).count();
    let communist_count = game.players.iter().filter(|p| p.role == Role::Communist).count();

    assert_eq!(liberal_count, 5, "10 players should have 5 Liberals");
    assert_eq!(fascist_count, 3, "10 players should have 3 Fascists");
    assert_eq!(hitler_count, 1, "10 players should have 1 Hitler");
    assert_eq!(communist_count, 1, "10 players should have 1 Communist");
}

#[test]
fn test_11_player_role_ratios() {
    // 11 players: 5L, 3F+H, 2C
    let opts = GameOptions {
        communists: true,
        monarchist: true,
        anarchist: true,
        capitalist: true,
        centrists: true,
        ..Default::default()
    };
    let player_names: Vec<String> = (0..11).map(|i| format!("Player{}", i)).collect();
    let game = Game::new(opts, &player_names, 42).unwrap();

    let liberal_count = game.players.iter().filter(|p| p.role == Role::Liberal).count();
    let fascist_count = game.players.iter().filter(|p| p.role == Role::Fascist).count();
    let hitler_count = game.players.iter().filter(|p| p.role == Role::Hitler).count();
    let communist_count = game.players.iter().filter(|p| p.role == Role::Communist).count();

    assert_eq!(liberal_count, 5, "11 players should have 5 Liberals");
    assert_eq!(fascist_count, 3, "11 players should have 3 Fascists");
    assert_eq!(hitler_count, 1, "11 players should have 1 Hitler");
    assert_eq!(communist_count, 2, "11 players should have 2 Communists");
}

#[test]
fn test_12_player_role_ratios() {
    // 12 players: 6L, 3F+H, 2C
    let opts = GameOptions {
        communists: true,
        monarchist: true,
        anarchist: true,
        capitalist: true,
        centrists: true,
        ..Default::default()
    };
    let player_names: Vec<String> = (0..12).map(|i| format!("Player{}", i)).collect();
    let game = Game::new(opts, &player_names, 42).unwrap();

    let liberal_count = game.players.iter().filter(|p| p.role == Role::Liberal).count();
    let fascist_count = game.players.iter().filter(|p| p.role == Role::Fascist).count();
    let hitler_count = game.players.iter().filter(|p| p.role == Role::Hitler).count();
    let communist_count = game.players.iter().filter(|p| p.role == Role::Communist).count();

    assert_eq!(liberal_count, 6, "12 players should have 6 Liberals");
    assert_eq!(fascist_count, 3, "12 players should have 3 Fascists");
    assert_eq!(hitler_count, 1, "12 players should have 1 Hitler");
    assert_eq!(communist_count, 2, "12 players should have 2 Communists");
}

#[test]
fn test_20_player_role_ratios() {
    // 20 players: 9L, 4F+H, 3C, 1M, 1A, 1Cap, 1Cen
    let opts = GameOptions {
        communists: true,
        monarchist: true,
        anarchist: true,
        capitalist: true,
        centrists: true,
        ..Default::default()
    };
    let player_names: Vec<String> = (0..20).map(|i| format!("Player{}", i)).collect();
    let game = Game::new(opts, &player_names, 42).unwrap();

    let liberal_count = game.players.iter().filter(|p| p.role == Role::Liberal).count();
    let fascist_count = game.players.iter().filter(|p| p.role == Role::Fascist).count();
    let hitler_count = game.players.iter().filter(|p| p.role == Role::Hitler).count();
    let communist_count = game.players.iter().filter(|p| p.role == Role::Communist).count();
    let monarchist_count = game.players.iter().filter(|p| p.role == Role::Monarchist).count();
    let anarchist_count = game.players.iter().filter(|p| p.role == Role::Anarchist).count();
    let capitalist_count = game.players.iter().filter(|p| p.role == Role::Capitalist).count();
    let centrist_count = game.players.iter().filter(|p| p.role == Role::Centrist).count();

    assert_eq!(liberal_count, 9, "20 players should have 9 Liberals");
    assert_eq!(fascist_count, 4, "20 players should have 4 Fascists");
    assert_eq!(hitler_count, 1, "20 players should have 1 Hitler");
    assert_eq!(communist_count, 3, "20 players should have 3 Communists");
    assert_eq!(monarchist_count, 1, "20 players should have 1 Monarchist");
    assert_eq!(anarchist_count, 1, "20 players should have 1 Anarchist");
    assert_eq!(capitalist_count, 1, "20 players should have 1 Capitalist");
    assert_eq!(centrist_count, 1, "20 players should have 1 Centrist");
}
