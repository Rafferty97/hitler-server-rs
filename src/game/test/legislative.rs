//! Legislative process tests

use super::super::{GameState, LegislativeSessionTurn, VetoStatus};
use super::test_utils::*;

#[test]
fn test_legislative_session_president_discard() {
    let mut game = create_game_in_legislative_session();

    if let GameState::LegislativeSession { president, turn, .. } = &game.state {
        let president = *president;

        if let LegislativeSessionTurn::President { cards } = turn {
            let original_cards = *cards;

            // President discards first card
            game.discard_policy(president, 0).unwrap();

            // Should advance to chancellor turn
            if let GameState::LegislativeSession { turn, .. } = &game.state {
                if let LegislativeSessionTurn::Chancellor { cards, .. } = turn {
                    assert_eq!(cards.len(), 2);
                    assert_eq!(*cards, [original_cards[1], original_cards[2]]);
                }
            }
        }
    }
}

#[test]
fn test_legislative_session_chancellor_enact() {
    let mut game = create_game_in_legislative_session();

    // Advance to chancellor turn
    if let GameState::LegislativeSession { president, .. } = &game.state {
        let president = *president;
        game.discard_policy(president, 0).unwrap();
    }

    // Chancellor enacts policy
    if let GameState::LegislativeSession { chancellor, turn, .. } = &game.state {
        let chancellor = *chancellor;

        if let LegislativeSessionTurn::Chancellor { cards, .. } = turn {
            let enacted_card = cards[0]; // Card that will be enacted (remaining after discard)
            game.discard_policy(chancellor, 1).unwrap(); // Discard the second card, enact the first

            // Should advance to card reveal
            if let GameState::CardReveal { result, .. } = &game.state {
                assert_eq!(*result, enacted_card);
            } else {
                // Test that the action was processed successfully
                assert!(matches!(game.state, GameState::CardReveal { .. }));
            }
        }
    }
}

#[test]
fn test_veto_mechanics() {
    let mut game = create_game_in_legislative_session();

    // Set up veto-enabled state
    game.board.fascist_cards = 5; // Veto unlocked

    // Advance to chancellor turn
    if let GameState::LegislativeSession { president, .. } = &game.state {
        let president = *president;
        game.discard_policy(president, 0).unwrap();
    }

    // Chancellor proposes veto
    if let GameState::LegislativeSession { chancellor, turn, .. } = &game.state {
        let chancellor = *chancellor;

        if let LegislativeSessionTurn::Chancellor { veto, .. } = turn {
            assert_eq!(*veto, VetoStatus::CanVeto);

            game.veto_agenda(chancellor).unwrap();

            // Should advance to veto requested state
            assert!(matches!(
                game.state,
                GameState::LegislativeSession {
                    turn: LegislativeSessionTurn::VetoRequested { .. },
                    ..
                }
            ));
        }
    }

    // President approves veto
    if let GameState::LegislativeSession { president, .. } = &game.state {
        let president = *president;
        game.veto_agenda(president).unwrap();

        // Should advance to veto approved
        assert!(matches!(
            game.state,
            GameState::LegislativeSession {
                turn: LegislativeSessionTurn::VetoApproved,
                ..
            }
        ));
    }
}

#[test]
fn test_veto_rejection() {
    let mut game = create_game_in_legislative_session();
    game.board.fascist_cards = 5; // Veto unlocked

    // Advance to veto requested state
    if let GameState::LegislativeSession { president, .. } = &game.state {
        let president = *president;
        game.discard_policy(president, 0).unwrap();
    }

    if let GameState::LegislativeSession { chancellor, .. } = &game.state {
        let chancellor = *chancellor;
        game.veto_agenda(chancellor).unwrap();
    }

    // President rejects veto
    if let GameState::LegislativeSession { president, .. } = &game.state {
        let president = *president;
        game.reject_veto(president).unwrap();

        // Should return to chancellor turn with veto denied
        if let GameState::LegislativeSession { turn, .. } = &game.state {
            if let LegislativeSessionTurn::Chancellor { veto, .. } = turn {
                assert_eq!(*veto, VetoStatus::VetoDenied);
            }
        }
    }
}

#[test]
fn test_invalid_card_index() {
    let mut game = create_game_in_legislative_session();

    if let GameState::LegislativeSession { president, .. } = &game.state {
        let president = *president;

        // Try to discard invalid card index
        let result = game.discard_policy(president, 5);
        assert!(result.is_err());
    }
}
