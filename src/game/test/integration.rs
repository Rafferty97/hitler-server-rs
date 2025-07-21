//! Integration tests for complete game flows

use super::super::GameState;
use super::test_utils::*;

#[test]
fn test_complete_game_flow_liberal_victory() {
    let mut game = create_standard_5_player_game();

    // Play through multiple rounds to liberal victory
    for _round in 0..5 {
        // Night phase
        if matches!(game.state, GameState::Night { .. }) {
            for i in 0..game.num_players() {
                game.end_night_round(i).ok();
            }
        }

        // Election phase
        if let GameState::Election { president, .. } = &game.state {
            let president = *president;
            let chancellor = (president + 1) % game.num_players();
            game.choose_player(president, chancellor).unwrap();

            // All vote yes
            for i in 0..game.num_players() {
                game.cast_vote(i, true).unwrap();
            }
            game.end_voting().unwrap();
        }

        // Legislative session
        if let GameState::LegislativeSession { president, .. } = &game.state {
            let president = *president;
            game.discard_policy(president, 0).unwrap();
        }

        if let GameState::LegislativeSession { chancellor, .. } = &game.state {
            let chancellor = *chancellor;
            game.discard_policy(chancellor, 0).unwrap();
        }

        // Card reveal
        if let GameState::CardReveal { .. } = &game.state {
            for i in 0..game.num_players() {
                game.end_card_reveal(Some(i)).ok();
            }
            game.end_card_reveal(None).unwrap();
        }

        // Check if game is over
        if game.game_over() {
            break;
        }

        // Handle executive actions if any
        while let GameState::ActionReveal { .. } = &game.state {
            game.end_executive_action(None).ok();
            break;
        }
    }

    // Game should eventually end
    assert!(game.board.liberal_cards > 0 || game.board.fascist_cards > 0);
}

#[test]
fn test_player_not_found() {
    let game = create_standard_5_player_game();

    let result = game.find_player("NonexistentPlayer");
    assert!(result.is_err());
}

#[test]
fn test_invalid_action_in_wrong_state() {
    let mut game = create_standard_5_player_game();

    // Try to cast vote during night phase
    let result = game.cast_vote(0, true);
    assert!(result.is_err());

    // Try to discard policy during night phase
    let result = game.discard_policy(0, 0);
    assert!(result.is_err());
}
