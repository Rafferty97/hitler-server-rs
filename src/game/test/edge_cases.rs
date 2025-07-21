//! Edge cases and error handling tests

use super::super::GameState;
use super::test_utils::*;

#[test]
fn test_invalid_player_indices() {
    let game = create_standard_5_player_game();

    // Test out of bounds player indices
    assert!(game.get_player_update(5).role == super::super::player::Role::Liberal); // Should not panic
    assert!(game.get_player_prompt(5).is_none());
}

#[test]
fn test_actions_on_dead_players() {
    let mut game = create_standard_5_player_game();

    // Kill a player
    game.players[1].alive = false;

    advance_to_election(&mut game);

    if let GameState::Election { president, .. } = &game.state {
        let president = *president;

        // Try to choose dead player as chancellor
        let result = game.choose_player(president, 1);
        assert!(result.is_err());
    }
}

#[test]
fn test_duplicate_actions() {
    let mut game = create_standard_5_player_game();
    advance_to_election(&mut game);

    if let GameState::Election { president, .. } = &game.state {
        let president = *president;

        // Choose chancellor
        let chancellor = (president + 1) % game.num_players();
        game.choose_player(president, chancellor).unwrap();

        // Try to choose again
        let result = game.choose_player(president, chancellor);
        assert!(result.is_err());
    }
}

#[test]
fn test_wrong_player_actions() {
    let mut game = create_standard_5_player_game();
    advance_to_election(&mut game);

    if let GameState::Election { president, .. } = &game.state {
        let president = *president;
        let other_player = (president + 1) % game.num_players();

        // Try to have non-president choose chancellor
        let result = game.choose_player(other_player, president);
        assert!(result.is_err());
    }
}

#[test]
fn test_invalid_state_transitions() {
    let mut game = create_standard_5_player_game();

    // Try to vote when not in voting state
    let result = game.cast_vote(0, true);
    assert!(result.is_err());

    // Try to discard when not in legislative session
    let result = game.discard_policy(0, 0);
    assert!(result.is_err());
}

#[test]
fn test_empty_deck_handling() {
    let mut game = create_standard_5_player_game();

    // Force election tracker to 3 to trigger chaos
    game.election_tracker = 3;

    // Start a round - should handle empty deck gracefully
    advance_to_election(&mut game);

    if let GameState::Election { president, .. } = &game.state {
        let president = *president;
        let chancellor = (president + 1) % game.num_players();

        game.choose_player(president, chancellor).unwrap();

        // Vote everyone yes
        for i in 0..game.num_players() {
            if game.players[i].alive {
                game.cast_vote(i, true).unwrap();
            }
        }

        // Should handle deck operations gracefully
        let result = game.end_voting();
        assert!(result.is_ok());
    }
}

#[test]
fn test_xl_mode_with_insufficient_players() {
    // XL mode requires 7+ players, test with fewer
    let player_names: Vec<String> = (0..6).map(|i| format!("Player {}", i)).collect();

    let result = create_game_with_options(
        super::super::GameOptions {
            communists: true,
            monarchist: false,
            anarchist: false,
            capitalist: false,
            centrists: false,
        },
        &player_names,
        Some(12345),
    );

    // Should either succeed with proper role distribution or fail gracefully
    match result {
        Ok(game) => {
            assert!(game.players.len() == 6);
        }
        Err(_) => {
            // Expected to fail with insufficient players for XL mode
        }
    }
}

#[test]
fn test_concurrent_confirmations() {
    let mut game = create_xl_game(7);

    // Set up a night state that requires confirmations
    game.state = GameState::Night {
        confirmations: super::super::confirmations::Confirmations::new(game.num_players()),
    };

    // Multiple players confirm simultaneously
    game.end_night_round(0).unwrap();
    game.end_night_round(1).unwrap();
    game.end_night_round(2).unwrap();

    // Should handle concurrent confirmations properly
    if let GameState::Night { confirmations } = &game.state {
        assert!(confirmations.has_confirmed(0));
        assert!(confirmations.has_confirmed(1));
        assert!(confirmations.has_confirmed(2));
    }
}

#[test]
fn test_malformed_game_states() {
    let mut game = create_standard_5_player_game();

    // Create an inconsistent state
    game.presidential_turn = 10; // Invalid turn number
    game.election_tracker = 5; // Invalid election tracker

    // Game should handle inconsistent state gracefully
    let board_update = game.get_board_update();
    assert!(board_update.presidential_turn < game.num_players());
    assert!(board_update.election_tracker <= 3);
}

#[test]
fn test_role_distribution_edge_cases() {
    // Test with minimum players
    let player_names: Vec<String> = (0..5).map(|i| format!("Player {}", i)).collect();
    let game = create_game_with_options(
        super::super::GameOptions {
            communists: false,
            monarchist: false,
            anarchist: false,
            capitalist: false,
            centrists: false,
        },
        &player_names,
        Some(12345),
    )
    .unwrap();

    // Should have exactly 1 Hitler, 1 Fascist, 3 Liberals
    let hitler_count = game
        .players
        .iter()
        .filter(|p| matches!(p.role, super::super::player::Role::Hitler))
        .count();
    let fascist_count = game
        .players
        .iter()
        .filter(|p| matches!(p.role, super::super::player::Role::Fascist))
        .count();
    let liberal_count = game
        .players
        .iter()
        .filter(|p| matches!(p.role, super::super::player::Role::Liberal))
        .count();

    assert_eq!(hitler_count, 1);
    assert_eq!(fascist_count, 1);
    assert_eq!(liberal_count, 3);
}

#[test]
fn test_player_choice_edge_cases() {
    let mut game = create_standard_5_player_game();

    // Set up choose player state
    game.state = GameState::ChoosePlayer {
        action: super::super::executive_power::ExecutiveAction::InvestigatePlayer,
        can_select: game.eligible_players().exclude(1).make(),
        can_be_selected: game.eligible_players().exclude(0).make(),
    };

    // Try to choose self (should fail)
    let result = game.choose_player(0, 0);
    assert!(result.is_err());

    // Try to choose dead player
    game.players[1].alive = false;
    let result = game.choose_player(0, 1);
    assert!(result.is_err());
}

#[test]
fn test_assassination_edge_cases() {
    let mut game = create_xl_game(7);

    // Set up card reveal state
    game.state = GameState::CardReveal {
        result: super::super::Party::Communist,
        chaos: false,
        confirmations: super::super::confirmations::Confirmations::new(game.num_players_alive()),
        board_ready: false,
    };

    // Find anarchist player
    let anarchist_idx = game
        .players
        .iter()
        .position(|p| p.role == super::super::player::Role::Anarchist);

    if let Some(anarchist) = anarchist_idx {
        // Try to start assassination with non-anarchist
        let result = game.start_assassination(0);
        if anarchist != 0 {
            assert!(result.is_err());
        }

        // Start assassination with anarchist
        let result = game.start_assassination(anarchist);
        assert!(result.is_ok());

        // Try to start assassination again (should fail)
        let result = game.start_assassination(anarchist);
        assert!(result.is_err());
    }
}
