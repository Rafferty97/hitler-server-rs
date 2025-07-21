//! Test utilities and helper functions for game testing

use super::super::confirmations::Confirmations;
use super::super::executive_power::ExecutiveAction;
use super::super::player::{Player, Role};
use super::super::votes::Votes;
use super::super::Party::*;
use super::super::{AssassinationState, GameState, LegislativeSessionTurn, VetoStatus, WinCondition};
use crate::game::deck::Deck;
use crate::game::government::Government;
use crate::game::{Game, GameOptions};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

/// Creates a test game with the specified number of players and options
pub fn create_test_game(num_players: usize, opts: GameOptions, seed: u64) -> Game {
    let player_names: Vec<String> = (0..num_players).map(|i| format!("Player{}", i)).collect();
    Game::new(opts, &player_names, seed).unwrap()
}

/// Creates a standard 5-player game for basic testing
pub fn create_standard_5_player_game() -> Game {
    create_test_game(5, GameOptions::default(), 42)
}

/// Creates an XL game with all features enabled
pub fn create_xl_game(num_players: usize) -> Game {
    let opts = GameOptions {
        communists: true,
        monarchist: true,
        anarchist: true,
        capitalist: true,
        centrists: true,
    };
    // Ensure we have enough players for XL mode with all features
    // Need at least 9 players for all XL features to work properly
    let actual_players = num_players.max(9);
    create_test_game(actual_players, opts, 42)
}

/// Helper to advance game to election state
pub fn advance_to_election(game: &mut Game) {
    if let GameState::Night { .. } = game.state {
        // End night round for all players
        for i in 0..game.num_players() {
            game.end_night_round(i).ok();
        }
    }
}

/// Helper to create a game in legislative session
pub fn create_game_in_legislative_session() -> Game {
    let mut game = create_standard_5_player_game();
    advance_to_election(&mut game);

    // Simulate successful election
    if let GameState::Election { president, .. } = &game.state {
        let president = *president;
        // Choose chancellor
        game.choose_player(president, (president + 1) % game.num_players())
            .unwrap();

        // All players vote yes
        for i in 0..game.num_players() {
            game.cast_vote(i, true).unwrap();
        }

        // End voting
        game.end_voting().unwrap();
    }

    game
}

/// Helper to set up a game with specific board state
pub fn create_game_with_board_state(liberal: usize, fascist: usize, communist: usize) -> Game {
    let mut game = create_xl_game(12); // Use 12 players for full XL configuration
    game.board.liberal_cards = liberal;
    game.board.fascist_cards = fascist;
    game.board.communist_cards = communist;
    game
}

/// Helper to create a game in a specific state for testing
pub fn create_game_in_state(state: GameState, num_players: usize) -> Game {
    let mut game = create_test_game(num_players, GameOptions::default(), 42);
    game.state = state;
    game
}

/// Creates a game with custom options
pub fn create_game_with_options(
    opts: GameOptions,
    player_names: &[String],
    seed: Option<u64>,
) -> Result<Game, super::super::GameError> {
    Game::new(opts, player_names, seed.unwrap_or(12345))
}
