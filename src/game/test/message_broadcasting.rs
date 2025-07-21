//! Message broadcasting tests

use super::super::GameState;
use super::test_utils::*;

#[test]
fn test_board_update_accuracy() {
    let game = create_standard_5_player_game();
    let board_update = game.get_board_update();

    assert_eq!(board_update.liberal_cards, game.board.liberal_cards);
    assert_eq!(board_update.fascist_cards, game.board.fascist_cards);
    assert_eq!(board_update.election_tracker, game.election_tracker);
    assert_eq!(board_update.presidential_turn, game.presidential_turn);
    assert_eq!(board_update.last_government, game.last_government);
    assert!(board_update.prompt.is_some());
}

#[test]
fn test_player_update_information_hiding() {
    let game = create_standard_5_player_game();

    for i in 0..game.num_players() {
        let player_update = game.get_player_update(i);

        // Player should know their own role
        assert_eq!(player_update.role, game.players[i].role);

        // Player should have investigation results for others
        assert_eq!(player_update.others.len(), game.num_players());

        // Check that hidden information is properly concealed
        for j in 0..game.num_players() {
            if i != j {
                let investigation = player_update.others[j];
                // Should either be unknown or revealed through game mechanics
                assert!(matches!(
                    investigation,
                    super::super::player::InvestigationResult::Unknown
                        | super::super::player::InvestigationResult::Party(_)
                        | super::super::player::InvestigationResult::Role(_)
                ));
            }
        }
    }
}

#[test]
fn test_public_player_information() {
    let mut game = create_standard_5_player_game();

    // Mark a player as not Hitler
    game.players[0].not_hitler = true;

    let public_players = game.get_public_players();

    assert_eq!(public_players.len(), game.num_players());

    for (i, public_player) in public_players.iter().enumerate() {
        assert_eq!(public_player.name, game.players[i].name);
        assert_eq!(public_player.alive, game.players[i].alive);
        assert_eq!(public_player.not_hitler, game.players[i].not_hitler);
    }
}

#[test]
fn test_player_prompt_accuracy() {
    let mut game = create_standard_5_player_game();
    advance_to_election(&mut game);

    if let GameState::Election { president, .. } = &game.state {
        let president = *president;

        // President should have chancellor nomination prompt
        let prompt = game.get_player_prompt(president);
        assert!(matches!(
            prompt,
            Some(super::super::update::PlayerPrompt::ChoosePlayer { .. })
        ));

        // Other players should not have prompts yet
        for i in 0..game.num_players() {
            if i != president {
                let prompt = game.get_player_prompt(i);
                assert!(prompt.is_none());
            }
        }
    }
}
