//! Victory condition tests

use super::super::confirmations::Confirmations;
use super::super::player::{Player, Role};
use super::super::Party::*;
use super::super::{GameState, LegislativeSessionTurn, WinCondition};
use super::test_utils::*;
use crate::game::deck::Deck;
use crate::game::{Game, GameOptions};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

#[test]
fn test_liberal_policy_track_victory() {
    let mut game = create_game_with_board_state(4, 0, 0);

    game.state = GameState::CardReveal {
        result: Liberal,
        chaos: false,
        confirmations: Confirmations::new(5),
        board_ready: false,
    };

    game.end_card_reveal(None).unwrap();

    assert!(game.game_over());
    assert_eq!(game.outcome(), Some(WinCondition::LiberalPolicyTrack));
}

#[test]
fn test_fascist_policy_track_victory() {
    let mut game = create_game_with_board_state(0, 5, 0);

    game.state = GameState::CardReveal {
        result: Fascist,
        chaos: false,
        confirmations: Confirmations::new(5),
        board_ready: false,
    };

    game.end_card_reveal(None).unwrap();

    assert!(game.game_over());
    assert_eq!(game.outcome(), Some(WinCondition::FascistPolicyTrack));
}

#[test]
fn test_communist_policy_track_victory() {
    let mut game = create_game_with_board_state(0, 0, 4);
    game.board.num_players = 7; // 5 communist policies needed for <8 players

    game.state = GameState::CardReveal {
        result: Communist,
        chaos: false,
        confirmations: Confirmations::new(5),
        board_ready: false,
    };

    game.end_card_reveal(None).unwrap();

    assert!(game.game_over());
    assert_eq!(game.outcome(), Some(WinCondition::CommunistPolicyTrack));
}

#[test]
fn test_hitler_chancellor_victory() {
    let mut game = create_standard_5_player_game();
    game.board.fascist_cards = 3; // Hitler chancellor victory enabled

    // Find Hitler
    let hitler_idx = game.players.iter().position(|p| p.role == Role::Hitler).unwrap();

    // Set up legislative session with Hitler as chancellor
    game.state = GameState::LegislativeSession {
        president: 0,
        chancellor: hitler_idx,
        turn: LegislativeSessionTurn::President { cards: [Liberal, Liberal, Liberal] },
    };

    // This should trigger game over check
    assert!(game.check_game_over());
    assert_eq!(game.outcome(), Some(WinCondition::HitlerChancellor));
}

#[test]
fn test_hitler_execution_victory() {
    let mut game = create_standard_5_player_game();

    // Find and kill Hitler
    let hitler_idx = game.players.iter().position(|p| p.role == Role::Hitler).unwrap();
    game.players[hitler_idx].alive = false;

    assert!(game.check_game_over());
    assert_eq!(game.outcome(), Some(WinCondition::HitlerExecuted));
}

#[test]
fn test_capitalist_execution_victory() {
    let mut game = create_xl_game(12);

    // Find and kill Capitalist
    let capitalist_idx = game.players.iter().position(|p| p.role == Role::Capitalist).unwrap();
    game.players[capitalist_idx].alive = false;

    assert!(game.check_game_over());
    assert_eq!(game.outcome(), Some(WinCondition::CapitalistExecuted));
}

// Original tests preserved
#[test]
fn liberal_track_victory() {
    let mut game = Game {
        opts: GameOptions::default(),
        board: super::super::board::Board {
            num_players: 5,
            liberal_cards: 4,
            fascist_cards: 0,
            communist_cards: 0,
        },
        deck: Deck::new(false),
        election_tracker: 0,
        last_government: None,
        players: vec![
            Player::new("ALEX".to_string(), Role::Liberal),
            Player::new("BOB".to_string(), Role::Liberal),
            Player::new("CHARLIE".to_string(), Role::Liberal),
            Player::new("DAVID".to_string(), Role::Liberal),
            Player::new("ED".to_string(), Role::Liberal),
        ],
        presidential_turn: 0,
        next_president: None,
        rng: ChaCha8Rng::seed_from_u64(0),
        state: GameState::CardReveal {
            result: Liberal,
            chaos: false,
            confirmations: Confirmations::new(5),
            board_ready: false,
        },
        radicalised: false,
        assassination: crate::game::AssassinationState::Unused,
    };

    game.end_card_reveal(None).unwrap();

    assert!(game.game_over());
    assert!(matches!(
        game.state,
        GameState::GameOver(WinCondition::LiberalPolicyTrack)
    ));
}

#[test]
fn fascist_track_victory() {
    let mut game = Game {
        opts: GameOptions::default(),
        board: super::super::board::Board {
            num_players: 5,
            liberal_cards: 0,
            fascist_cards: 5,
            communist_cards: 0,
        },
        deck: Deck::new(false),
        election_tracker: 0,
        last_government: None,
        players: vec![
            Player::new("ALEX".to_string(), Role::Liberal),
            Player::new("BOB".to_string(), Role::Liberal),
            Player::new("CHARLIE".to_string(), Role::Liberal),
            Player::new("DAVID".to_string(), Role::Liberal),
            Player::new("ED".to_string(), Role::Liberal),
        ],
        presidential_turn: 0,
        next_president: None,
        rng: ChaCha8Rng::seed_from_u64(0),
        state: GameState::CardReveal {
            result: Fascist,
            chaos: false,
            confirmations: Confirmations::new(5),
            board_ready: false,
        },
        radicalised: false,
        assassination: crate::game::AssassinationState::Unused,
    };

    game.end_card_reveal(None).unwrap();

    assert!(game.game_over());
    assert!(matches!(
        game.state,
        GameState::GameOver(WinCondition::FascistPolicyTrack)
    ));
}
