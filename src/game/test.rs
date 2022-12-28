#![cfg(test)]

use super::confirmations::Confirmations;
use super::player::Player;
use super::player::Role;
use super::GameState;
use super::Party::*;
use crate::game::government::Government;
use crate::game::Game;
use crate::game::WinCondition;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

#[test]
fn can_create_game() {
    let players = ["Alex", "Bob", "Charlie", "David", "Ed"].map(|s| s.into());
    let game = Game::new(&players, 0);
    assert!(matches!(game.state, GameState::Night { .. }));
}

#[test]
fn liberal_track_victory() {
    let mut game = Game {
        board: super::board::Board {
            num_players: 5,
            liberal_cards: 4,
            fascist_cards: 0,
        },
        deck: vec![Liberal, Liberal, Liberal, Liberal, Liberal],
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
        rng: ChaCha8Rng::seed_from_u64(0),
        state: GameState::CardReveal {
            result: Liberal,
            chaos: false,
            confirmations: Confirmations::new(5),
            board_ready: false,
        },
    };

    game.end_card_reveal(None).unwrap();

    assert!(game.game_over());
    assert!(matches!(
        game.state,
        GameState::GameOver {
            winner: Liberal,
            win_condition: WinCondition::Legislative
        }
    ));
}

#[test]
fn fascist_track_victory() {
    let mut game = Game {
        board: super::board::Board {
            num_players: 5,
            liberal_cards: 0,
            fascist_cards: 5,
        },
        deck: vec![Liberal, Liberal, Liberal, Liberal, Liberal],
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
        rng: ChaCha8Rng::seed_from_u64(0),
        state: GameState::CardReveal {
            result: Fascist,
            chaos: false,
            confirmations: Confirmations::new(5),
            board_ready: false,
        },
    };

    game.end_card_reveal(None).unwrap();

    assert!(game.game_over());
    assert!(matches!(
        game.state,
        GameState::GameOver {
            winner: Fascist,
            win_condition: WinCondition::Legislative
        }
    ));
}

#[test]
fn eligible_chancellors_5players() {
    let mut game = Game {
        board: super::board::Board {
            num_players: 5,
            liberal_cards: 0,
            fascist_cards: 0,
        },
        deck: vec![Liberal, Liberal, Liberal, Liberal, Liberal],
        election_tracker: 0,
        last_government: Some(Government {
            president: 0,
            chancellor: 3,
        }),
        players: vec![
            Player::new("ALEX".to_string(), Role::Liberal),
            Player::new("BOB".to_string(), Role::Liberal),
            Player::new("CHARLIE".to_string(), Role::Liberal),
            Player::new("DAVID".to_string(), Role::Fascist),
            Player::new("ED".to_string(), Role::Hitler),
        ],
        presidential_turn: 0,
        rng: ChaCha8Rng::seed_from_u64(0),
        state: GameState::CardReveal {
            result: Fascist,
            chaos: false,
            confirmations: Confirmations::new(5),
            board_ready: false,
        },
    };

    for i in 0..5 {
        game.end_card_reveal(Some(i)).unwrap();
    }
    game.end_card_reveal(None).unwrap();

    let GameState::Election { president, chancellor, eligible_chancellors, votes } = game.state else {
        panic!("Expected an election");
    };

    assert_eq!(president, 1);
    assert_eq!(chancellor, None);
    assert_eq!(
        &eligible_chancellors[0..5],
        &[true, false, true, false, true]
    );
    assert_eq!(votes.outcome(), None);
}
