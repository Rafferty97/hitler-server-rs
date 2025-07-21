//! Communist powers tests (XL Expansion)

use super::super::executive_power::ExecutiveAction;
use super::super::GameState;
use super::super::Party::*;
use super::test_utils::*;
use crate::game::government::Government;

#[test]
fn test_bugging_power() {
    let mut game = create_game_with_board_state(0, 0, 1); // 1st communist policy

    game.last_government = Some(Government { president: 0, chancellor: 1 });

    let action = game.board.get_executive_power(Communist);
    assert_eq!(action, Some(ExecutiveAction::Bugging));

    game.start_executive_action(ExecutiveAction::Bugging);

    // Should enter communist session
    assert!(matches!(
        game.state,
        GameState::CommunistStart { action: ExecutiveAction::Bugging }
    ));
}

#[test]
fn test_radicalisation_power() {
    let mut game = create_game_with_board_state(0, 0, 2); // 2nd communist policy

    game.last_government = Some(Government { president: 0, chancellor: 1 });

    let action = game.board.get_executive_power(Communist);
    assert_eq!(action, Some(ExecutiveAction::Radicalisation));

    game.start_executive_action(ExecutiveAction::Radicalisation);

    assert!(matches!(
        game.state,
        GameState::CommunistStart { action: ExecutiveAction::Radicalisation }
    ));
}

#[test]
fn test_five_year_plan_power() {
    let mut game = create_game_with_board_state(0, 0, 3); // 3rd communist policy

    game.last_government = Some(Government { president: 0, chancellor: 1 });

    let action = game.board.get_executive_power(Communist);
    assert_eq!(action, Some(ExecutiveAction::FiveYearPlan));

    game.start_executive_action(ExecutiveAction::FiveYearPlan);

    // Should be in action reveal state
    assert!(matches!(
        game.state,
        GameState::ActionReveal { action: ExecutiveAction::FiveYearPlan, .. }
    ));
}

#[test]
fn test_congress_power() {
    let mut game = create_game_with_board_state(0, 0, 4); // 4th communist policy

    game.last_government = Some(Government { president: 0, chancellor: 1 });

    let action = game.board.get_executive_power(Communist);
    assert_eq!(action, Some(ExecutiveAction::Congress));

    game.start_executive_action(ExecutiveAction::Congress);

    assert!(matches!(
        game.state,
        GameState::CommunistStart { action: ExecutiveAction::Congress }
    ));
}

#[test]
fn test_confession_power() {
    let mut game = create_game_with_board_state(0, 0, 5); // 5th communist policy in 8+ players
    game.board.num_players = 8;

    game.last_government = Some(Government { president: 0, chancellor: 1 });

    let action = game.board.get_executive_power(Communist);
    assert_eq!(action, Some(ExecutiveAction::Confession));

    game.start_executive_action(ExecutiveAction::Confession);

    // Should be in choose player state with chancellor selecting
    if let GameState::ChoosePlayer { action, can_select, .. } = &game.state {
        assert_eq!(*action, ExecutiveAction::Confession);
        assert!(can_select.includes(1)); // Chancellor can select
        assert!(!can_select.includes(0)); // President cannot select
    }
}
