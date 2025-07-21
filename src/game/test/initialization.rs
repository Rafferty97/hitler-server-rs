//! Game initialization and setup tests

use super::super::player::Role;
use super::super::GameState;
use super::test_utils::*;
use crate::game::{Game, GameOptions};

#[test]
fn test_game_creation_standard() {
    let players = ["Alice", "Bob", "Charlie", "David", "Eve"].map(|s| s.into());
    let opts = GameOptions::default();
    let game = Game::new(opts, &players, 0).unwrap();

    assert_eq!(game.num_players(), 5);
    assert!(matches!(game.state, GameState::Night { .. }));
    assert_eq!(game.election_tracker, 0);
    assert_eq!(game.board.liberal_cards, 0);
    assert_eq!(game.board.fascist_cards, 0);
    assert_eq!(game.board.communist_cards, 0);
}

#[test]
fn test_game_creation_xl_features() {
    let opts = GameOptions {
        communists: true,
        monarchist: true,
        anarchist: true,
        capitalist: true,
        centrists: true,
    };
    let players: Vec<String> = (0..10).map(|i| format!("Player{}", i)).collect();
    let game = Game::new(opts, &players, 0).unwrap();

    assert_eq!(game.num_players(), 10);
    assert!(game.options().communists);
    assert!(game.options().monarchist);
    assert!(game.options().anarchist);
    assert!(game.options().capitalist);
    assert!(game.options().centrists);
}

#[test]
fn test_invalid_player_counts() {
    let opts = GameOptions::default();

    // Too few players
    let players = ["Alice", "Bob", "Charlie"].map(|s| s.into());
    assert!(Game::new(opts, &players, 0).is_err());

    // Too many players for standard game
    let players: Vec<String> = (0..20).map(|i| format!("Player{}", i)).collect();
    assert!(Game::new(opts, &players, 0).is_err());
}

#[test]
fn test_role_distribution_standard_5_players() {
    let game = create_standard_5_player_game();

    let liberal_count = game.players.iter().filter(|p| p.role == Role::Liberal).count();
    let fascist_count = game.players.iter().filter(|p| p.role == Role::Fascist).count();
    let hitler_count = game.players.iter().filter(|p| p.role == Role::Hitler).count();

    assert_eq!(liberal_count, 3);
    assert_eq!(fascist_count, 1);
    assert_eq!(hitler_count, 1);
}

#[test]
fn test_role_distribution_xl_10_players() {
    let game = create_xl_game(10);

    let roles: Vec<Role> = game.players.iter().map(|p| p.role).collect();

    // Count each role
    let _liberal_count = roles.iter().filter(|&&r| r == Role::Liberal).count();
    let fascist_count = roles.iter().filter(|&&r| r == Role::Fascist).count();
    let communist_count = roles.iter().filter(|&&r| r == Role::Communist).count();
    let hitler_count = roles.iter().filter(|&&r| r == Role::Hitler).count();
    let monarchist_count = roles.iter().filter(|&&r| r == Role::Monarchist).count();
    let anarchist_count = roles.iter().filter(|&&r| r == Role::Anarchist).count();
    let capitalist_count = roles.iter().filter(|&&r| r == Role::Capitalist).count();
    let centrist_count = roles.iter().filter(|&&r| r == Role::Centrist).count();

    assert_eq!(hitler_count, 1);
    assert_eq!(monarchist_count, 1);
    assert_eq!(anarchist_count, 1);
    assert_eq!(capitalist_count, 1);
    assert_eq!(centrist_count, 2);
    // liberal_count is usize, so always >= 0
    assert!(fascist_count >= 1);
    assert!(communist_count >= 1);

    // Total should equal player count
    assert_eq!(roles.len(), 10);
}

#[test]
fn test_role_revelation_fascists_know_each_other() {
    let game = create_standard_5_player_game();

    // Find fascist players
    let fascist_indices: Vec<usize> = game
        .players
        .iter()
        .enumerate()
        .filter(|(_, p)| p.role == Role::Fascist || p.role == Role::Hitler)
        .map(|(i, _)| i)
        .collect();

    // Each fascist should know the others
    for &fascist_idx in &fascist_indices {
        for &other_fascist_idx in &fascist_indices {
            if fascist_idx != other_fascist_idx {
                let investigation = game.players[fascist_idx].others[other_fascist_idx];
                assert!(matches!(
                    investigation,
                    super::super::player::InvestigationResult::Role(_)
                ));
            }
        }
    }
}

#[test]
fn test_game_with_minimum_players() {
    let opts = GameOptions::default();
    let min_players = opts.min_players().unwrap();

    let players: Vec<String> = (0..min_players).map(|i| format!("Player{}", i)).collect();
    let game = Game::new(opts, &players, 0);

    assert!(game.is_ok());
}

#[test]
fn test_game_with_maximum_players() {
    let opts = GameOptions::default();
    let max_players = opts.max_players().unwrap();

    let players: Vec<String> = (0..max_players).map(|i| format!("Player{}", i)).collect();
    let game = Game::new(opts, &players, 0);

    assert!(game.is_ok());
}

#[test]
fn test_xl_game_with_minimum_players() {
    let opts = GameOptions {
        communists: true,
        monarchist: true,
        anarchist: true,
        capitalist: true,
        centrists: true,
    };
    let min_players = opts.min_players().unwrap();

    let players: Vec<String> = (0..min_players).map(|i| format!("Player{}", i)).collect();
    let game = Game::new(opts, &players, 0);

    assert!(game.is_ok());
}

#[test]
fn test_xl_game_with_maximum_players() {
    let opts = GameOptions {
        communists: true,
        monarchist: true,
        anarchist: true,
        capitalist: true,
        centrists: true,
    };
    let max_players = opts.max_players().unwrap();

    let players: Vec<String> = (0..max_players).map(|i| format!("Player{}", i)).collect();
    let game = Game::new(opts, &players, 0);

    assert!(game.is_ok());
}

#[test]
fn test_deterministic_behavior_with_same_seed() {
    let players = ["Alice", "Bob", "Charlie", "David", "Eve"].map(|s| s.into());
    let opts = GameOptions::default();

    let game1 = Game::new(opts, &players, 12345).unwrap();
    let game2 = Game::new(opts, &players, 12345).unwrap();

    // Games with same seed should have identical initial state
    assert_eq!(game1.presidential_turn, game2.presidential_turn);

    // Player roles should be identical
    for i in 0..game1.num_players() {
        assert_eq!(game1.players[i].role, game2.players[i].role);
    }
}

#[test]
fn test_different_behavior_with_different_seeds() {
    let players = ["Alice", "Bob", "Charlie", "David", "Eve"].map(|s| s.into());
    let opts = GameOptions::default();

    let game1 = Game::new(opts, &players, 12345).unwrap();
    let game2 = Game::new(opts, &players, 54321).unwrap();

    // Games with different seeds should likely have different initial state
    // (This might occasionally fail due to randomness, but very unlikely)
    let roles_different = (0..game1.num_players()).any(|i| game1.players[i].role != game2.players[i].role);

    assert!(roles_different || game1.presidential_turn != game2.presidential_turn);
}

// Original test preserved
#[test]
fn can_create_game() {
    let players = ["Alex", "Bob", "Charlie", "David", "Ed"].map(|s| s.into());
    let opts = GameOptions::default();
    let game = Game::new(opts, &players, 0).unwrap();
    assert!(matches!(game.state, GameState::Night { .. }));
}
