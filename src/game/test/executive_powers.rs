//! Executive powers tests (Fascist powers)

use super::super::executive_power::ExecutiveAction;
use super::super::player::Role;
use super::super::GameState;
use super::super::Party::*;
use super::test_utils::*;
use crate::game::government::Government;

#[test]
fn test_policy_peek_power() {
    let mut game = create_game_with_board_state(0, 2, 0); // 3rd fascist policy triggers peek in 5-6 players
    game.board.num_players = 6; // Use 6 players for policy peek
    game.board.fascist_cards = 3; // Set to 3 for policy peek power

    // Set up last government
    game.last_government = Some(Government { president: 0, chancellor: 1 });

    // Trigger policy peek
    let action = game.board.get_executive_power(Fascist);
    assert_eq!(action, Some(ExecutiveAction::PolicyPeak));

    game.start_executive_action(ExecutiveAction::PolicyPeak);

    // Should be in action reveal state
    assert!(matches!(
        game.state,
        GameState::ActionReveal { action: ExecutiveAction::PolicyPeak, .. }
    ));
}

#[test]
fn test_investigate_player_power() {
    let mut game = create_game_with_board_state(0, 0, 0); // 1st fascist policy in 9-10 players
    game.board.num_players = 9;
    game.board.fascist_cards = 1; // Set to 1 for investigate power

    game.last_government = Some(Government { president: 0, chancellor: 1 });

    let action = game.board.get_executive_power(Fascist);
    assert_eq!(action, Some(ExecutiveAction::InvestigatePlayer));

    game.start_executive_action(ExecutiveAction::InvestigatePlayer);

    // Should be in choose player state
    if let GameState::ChoosePlayer { action, can_select, can_be_selected } = &game.state {
        assert_eq!(*action, ExecutiveAction::InvestigatePlayer);
        assert!(can_select.includes(0)); // President can select
        assert!(!can_be_selected.includes(0)); // President cannot be selected
    }
}

#[test]
fn test_special_election_power() {
    let mut game = create_game_with_board_state(0, 2, 0); // 3rd fascist policy in 7-10 players
    game.board.num_players = 8; // Use 8 players for special election
    game.board.fascist_cards = 3; // Set to 3 for special election power

    game.last_government = Some(Government { president: 0, chancellor: 1 });

    let action = game.board.get_executive_power(Fascist);
    assert_eq!(action, Some(ExecutiveAction::SpecialElection));

    game.start_executive_action(ExecutiveAction::SpecialElection);

    // Should check for monarchist first
    if game.players.iter().any(|p| p.role == Role::Monarchist) {
        assert!(matches!(game.state, GameState::PromptMonarchist { .. }));
    } else {
        assert!(matches!(
            game.state,
            GameState::ChoosePlayer { action: ExecutiveAction::SpecialElection, .. }
        ));
    }
}

#[test]
fn test_execution_power() {
    let mut game = create_game_with_board_state(0, 3, 0); // 4th fascist policy
    game.board.fascist_cards = 4; // Set to 4 for execution power

    game.last_government = Some(Government { president: 0, chancellor: 1 });

    let action = game.board.get_executive_power(Fascist);
    assert_eq!(action, Some(ExecutiveAction::Execution));

    game.start_executive_action(ExecutiveAction::Execution);

    // Should be in choose player state
    if let GameState::ChoosePlayer { action, can_select, can_be_selected } = &game.state {
        assert_eq!(*action, ExecutiveAction::Execution);
        assert!(can_select.includes(0)); // President can select
        assert!(!can_be_selected.includes(0)); // President cannot be selected
    }
}
