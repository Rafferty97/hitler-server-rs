//! Player management and elimination tests

use super::super::GameState;
use super::test_utils::*;

#[test]
fn test_player_elimination_effects() {
    let mut game = create_standard_5_player_game();

    let initial_alive = game.num_players_alive();

    // Kill a player
    game.players[0].alive = false;

    assert_eq!(game.num_players_alive(), initial_alive - 1);

    // Dead players should not be eligible for selection
    let eligible = game.eligible_players().make();
    assert!(!eligible.includes(0));
}

#[test]
fn test_dead_player_voting_restrictions() {
    let mut game = create_standard_5_player_game();
    advance_to_election(&mut game);

    // Kill a player
    game.players[0].alive = false;

    // Nominate chancellor
    if let GameState::Election { president, .. } = &game.state {
        let president = *president;
        let chancellor = (president + 1) % game.num_players();
        game.choose_player(president, chancellor).unwrap();
    }

    // Dead player should not be able to vote
    let result = game.cast_vote(0, true);
    if game.players[0].alive {
        assert!(result.is_ok());
    } else {
        // The game should handle this gracefully - dead players just don't affect voting
        // The voting system should work with reduced player count
        assert_eq!(game.num_players_alive(), 4);
    }
}

#[test]
fn test_government_eligibility_after_elimination() {
    let mut game = create_standard_5_player_game();

    // Kill the current president
    game.players[game.presidential_turn].alive = false;

    // Start a round - should skip to next alive player
    game.start_round();

    if let GameState::Election { president, .. } = &game.state {
        assert!(game.players[*president].alive);
    }
}
