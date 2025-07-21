//! Special roles tests (XL Expansion)

use super::super::confirmations::Confirmations;
use super::super::player::Role;
use super::super::Party::*;
use super::super::{AssassinationState, GameState};
use super::test_utils::*;

#[test]
fn test_anarchist_assassination() {
    let mut game = create_xl_game(12);

    // Find anarchist
    let anarchist_idx = game.players.iter().position(|p| p.role == Role::Anarchist).unwrap();

    // Set up card reveal state
    game.state = GameState::CardReveal {
        result: Liberal,
        chaos: false,
        confirmations: Confirmations::new(game.num_players_alive()),
        board_ready: false,
    };

    // Anarchist starts assassination
    game.start_assassination(anarchist_idx).unwrap();

    assert_eq!(
        game.assassination,
        AssassinationState::Activated { anarchist: anarchist_idx }
    );
}

#[test]
fn test_monarchist_special_election_hijack() {
    let mut game = create_xl_game(12);

    // Find monarchist
    let monarchist_idx = game.players.iter().position(|p| p.role == Role::Monarchist).unwrap();

    // Set up special election prompt
    game.state = GameState::PromptMonarchist {
        monarchist: monarchist_idx,
        last_president: 0,
        hijacked: false,
    };

    // Monarchist hijacks election
    game.hijack_special_election(monarchist_idx).unwrap();

    if let GameState::PromptMonarchist { hijacked, .. } = &game.state {
        assert!(*hijacked);
    }
}

#[test]
fn test_centrist_radicalisation() {
    let mut game = create_xl_game(12);

    // Find centrist
    let centrist_idx = game.players.iter().position(|p| p.role == Role::Centrist).unwrap();

    // Attempt radicalisation
    let success = game.players[centrist_idx].radicalise();

    assert!(success);
    assert_eq!(game.players[centrist_idx].role, Role::Communist);
    assert!(game.players[centrist_idx].tried_to_radicalise);
}

#[test]
fn test_liberal_radicalisation() {
    let mut game = create_xl_game(12);

    // Find liberal
    let liberal_idx = game.players.iter().position(|p| p.role == Role::Liberal).unwrap();

    // Attempt radicalisation
    let success = game.players[liberal_idx].radicalise();

    assert!(success);
    assert_eq!(game.players[liberal_idx].role, Role::Communist);
    assert!(game.players[liberal_idx].tried_to_radicalise);
}

#[test]
fn test_fascist_radicalisation_fails() {
    let mut game = create_xl_game(12);

    // Find fascist
    let fascist_idx = game.players.iter().position(|p| p.role == Role::Fascist).unwrap();
    let original_role = game.players[fascist_idx].role;

    // Attempt radicalisation
    let success = game.players[fascist_idx].radicalise();

    assert!(!success);
    assert_eq!(game.players[fascist_idx].role, original_role);
    assert!(game.players[fascist_idx].tried_to_radicalise);
}
