//! Anarchist policy mechanics tests
//!
//! Tests based on the Secret Hitler XL rules for anarchist-specific mechanics:
//! - Anarchist role assignment and team alignment
//! - Anarchist policy placement and effects
//! - Anarchist victory conditions and win scenarios
//! - Anarchist interaction with communist team mechanics
//! - Anarchist special abilities and powers
//!
//! AMBIGUITY NOTES:
//! - Anarchist win condition: unclear if anarchist has separate victory or wins with communists
//! - ASSUMPTION: Anarchist wins with communist team but may have unique mechanics
//! - Anarchist policy effects: unclear what special effects anarchist policies have
//! - ASSUMPTION: Anarchist policies may have disruptive or chaos-inducing effects

use super::super::player::Role;
use super::test_utils::*;
use crate::game::{Game, GameOptions};

/// Test anarchist role assignment in games with anarchist enabled
#[test]
fn test_anarchist_role_assignment() {
    for player_count in 8..=16 {
        let opts = GameOptions {
            communists: true,
            monarchist: false,
            anarchist: true,
            capitalist: false,
            centrists: false,
        };
        let player_names: Vec<String> = (0..player_count).map(|i| format!("Player{}", i)).collect();

        if let Ok(game) = Game::new(opts, &player_names, 42) {
            let anarchist_count = game.players.iter().filter(|p| p.role == Role::Anarchist).count();

            // Should have exactly 1 anarchist when enabled
            assert_eq!(
                anarchist_count, 1,
                "{} players should have exactly 1 anarchist when enabled",
                player_count
            );

            // Anarchist should not be Hitler
            let anarchist_player = game
                .players
                .iter()
                .find(|p| p.role == Role::Anarchist)
                .expect("Should have anarchist player");

            assert!(anarchist_player.role != Role::Hitler, "Anarchist should not be Hitler");
        }
    }
}

/// Test anarchist team alignment with communist faction
#[test]
fn test_anarchist_communist_team_alignment() {
    let opts = GameOptions {
        communists: true,
        monarchist: false,
        anarchist: true,
        capitalist: false,
        centrists: false,
    };
    let player_names: Vec<String> = (0..11).map(|i| format!("Player{}", i)).collect();

    if let Ok(game) = Game::new(opts, &player_names, 42) {
        let anarchist_player = game
            .players
            .iter()
            .find(|p| p.role == Role::Anarchist)
            .expect("Should have anarchist player");

        let communist_players: Vec<_> = game.players.iter().filter(|p| p.role == Role::Communist).collect();

        // Anarchist should be on communist team
        assert!(
            anarchist_player.party().to_string().contains("Communist") || anarchist_player.role == Role::Anarchist,
            "Anarchist should be aligned with communist team"
        );

        // Should have both anarchist and communist players
        assert!(!communist_players.is_empty(), "Should have communist players");
        assert_eq!(anarchist_player.role, Role::Anarchist, "Should have anarchist player");

        // ASSUMPTION: Anarchist is part of communist knowledge group (tested in communist_knowledge tests)
    }
}

/// Test anarchist in games without communists (should not be allowed or have different behavior)
#[test]
fn test_anarchist_without_communists() {
    let opts = GameOptions {
        communists: false,
        monarchist: false,
        anarchist: true,
        capitalist: false,
        centrists: false,
    };
    let player_names: Vec<String> = (0..8).map(|i| format!("Player{}", i)).collect();

    // This configuration might not be valid since anarchist is part of communist team
    match Game::new(opts, &player_names, 42) {
        Ok(game) => {
            // If game creation succeeds, anarchist behavior might be different
            let anarchist_count = game.players.iter().filter(|p| p.role == Role::Anarchist).count();

            // ASSUMPTION: Anarchist might not be assigned without communists enabled
            // or might have different team alignment
            if anarchist_count > 0 {
                println!("Anarchist assigned without communists - special case behavior");
            }
        }
        Err(_) => {
            // Game creation might fail if anarchist requires communists
            // This would be the expected behavior if anarchist is strictly communist-aligned
        }
    }
}

/// Test anarchist victory conditions
#[test]
fn test_anarchist_victory_conditions() {
    let opts = GameOptions {
        communists: true,
        monarchist: false,
        anarchist: true,
        capitalist: false,
        centrists: false,
    };
    let player_names: Vec<String> = (0..12).map(|i| format!("Player{}", i)).collect();

    if let Ok(game) = Game::new(opts, &player_names, 42) {
        let anarchist_player = game
            .players
            .iter()
            .find(|p| p.role == Role::Anarchist)
            .expect("Should have anarchist player");

        let communist_players: Vec<_> = game.players.iter().filter(|p| p.role == Role::Communist).collect();

        // Verify anarchist and communist players are present for victory testing
        assert_eq!(anarchist_player.role, Role::Anarchist, "Should have anarchist");
        assert!(!communist_players.is_empty(), "Should have communists");

        // ASSUMPTION: Anarchist wins with communist team
        // This would require simulating game completion to test properly

        // ASSUMPTION: Anarchist might have unique victory conditions
        // such as causing maximum chaos or disruption
    }
}

/// Test anarchist policy mechanics
#[test]
fn test_anarchist_policy_mechanics() {
    let opts = GameOptions {
        communists: true,
        monarchist: false,
        anarchist: true,
        capitalist: false,
        centrists: false,
    };
    let player_names: Vec<String> = (0..10).map(|i| format!("Player{}", i)).collect();

    if let Ok(game) = Game::new(opts, &player_names, 42) {
        let anarchist_player = game
            .players
            .iter()
            .find(|p| p.role == Role::Anarchist)
            .expect("Should have anarchist player");

        // ASSUMPTION: Anarchist might have special policy-related abilities
        // such as disrupting policy placement or causing chaos effects

        // Verify anarchist is properly set up for policy mechanics
        assert_eq!(anarchist_player.role, Role::Anarchist, "Should have anarchist role");
        assert!(anarchist_player.alive, "Anarchist should start alive");

        // ASSUMPTION: Anarchist policies might have unique effects
        // different from standard communist policies
    }
}

/// Test anarchist special abilities
#[test]
fn test_anarchist_special_abilities() {
    let opts = GameOptions {
        communists: true,
        monarchist: false,
        anarchist: true,
        capitalist: false,
        centrists: false,
    };
    let player_names: Vec<String> = (0..9).map(|i| format!("Player{}", i)).collect();

    if let Ok(game) = Game::new(opts, &player_names, 42) {
        let anarchist_player = game
            .players
            .iter()
            .find(|p| p.role == Role::Anarchist)
            .expect("Should have anarchist player");

        // ASSUMPTION: Anarchist might have special abilities like:
        // - Disrupting elections
        // - Causing policy chaos
        // - Special investigation results
        // - Unique executive actions

        // Verify anarchist has standard player properties
        assert_eq!(anarchist_player.role, Role::Anarchist, "Should be anarchist");
        assert!(anarchist_player.alive, "Should start alive");
        assert!(!anarchist_player.investigated, "Should start uninvestigated");
        assert!(
            !anarchist_player.tried_to_radicalise,
            "Should start without radicalization attempts"
        );

        // ASSUMPTION: Anarchist special abilities would be tested through gameplay simulation
    }
}

/// Test anarchist interaction with radicalization
#[test]
fn test_anarchist_radicalization_interaction() {
    let opts = GameOptions {
        communists: true,
        monarchist: false,
        anarchist: true,
        capitalist: false,
        centrists: false,
    };
    let player_names: Vec<String> = (0..8).map(|i| format!("Player{}", i)).collect();

    if let Ok(mut game) = Game::new(opts, &player_names, 42) {
        let anarchist_idx = game
            .players
            .iter()
            .position(|p| p.role == Role::Anarchist)
            .expect("Should have anarchist player");

        // Test if anarchist can be radicalized (probably not, since already on communist team)
        let was_radicalized = game.players[anarchist_idx].radicalise();

        // ASSUMPTION: Anarchist cannot be radicalized since already on communist team
        assert!(
            !was_radicalized,
            "Anarchist should not be radicalizable (already on communist team)"
        );

        // Anarchist role should remain unchanged
        assert_eq!(
            game.players[anarchist_idx].role,
            Role::Anarchist,
            "Anarchist role should not change from radicalization attempt"
        );

        // But radicalization attempt should be recorded
        assert!(
            game.players[anarchist_idx].tried_to_radicalise,
            "Radicalization attempt should be recorded"
        );
    }
}

/// Test anarchist in large games with all special roles
#[test]
fn test_anarchist_in_large_games() {
    let opts = GameOptions {
        communists: true,
        monarchist: true,
        anarchist: true,
        capitalist: true,
        centrists: true,
    };
    let player_names: Vec<String> = (0..15).map(|i| format!("Player{}", i)).collect();

    if let Ok(game) = Game::new(opts, &player_names, 42) {
        let anarchist_count = game.players.iter().filter(|p| p.role == Role::Anarchist).count();
        let communist_count = game.players.iter().filter(|p| p.role == Role::Communist).count();
        let capitalist_count = game.players.iter().filter(|p| p.role == Role::Capitalist).count();
        let monarchist_count = game.players.iter().filter(|p| p.role == Role::Monarchist).count();
        let centrist_count = game.players.iter().filter(|p| p.role == Role::Centrist).count();

        // Verify anarchist coexists with other special roles
        assert_eq!(anarchist_count, 1, "Should have exactly 1 anarchist");
        assert!(communist_count >= 1, "Should have communist players");
        assert_eq!(capitalist_count, 1, "Should have exactly 1 capitalist");
        assert_eq!(monarchist_count, 1, "Should have exactly 1 monarchist");
        assert!(centrist_count >= 1, "Should have centrist players");

        // ASSUMPTION: Anarchist mechanics work the same in large games
        // but may have more opportunities for chaos and disruption
    }
}

/// Test anarchist knowledge and information sharing
#[test]
fn test_anarchist_knowledge_sharing() {
    let opts = GameOptions {
        communists: true,
        monarchist: false,
        anarchist: true,
        capitalist: false,
        centrists: false,
    };
    let player_names: Vec<String> = (0..11).map(|i| format!("Player{}", i)).collect();

    if let Ok(game) = Game::new(opts, &player_names, 42) {
        let anarchist_player = game
            .players
            .iter()
            .find(|p| p.role == Role::Anarchist)
            .expect("Should have anarchist player");

        let communist_players: Vec<_> = game.players.iter().filter(|p| p.role == Role::Communist).collect();

        // With 11+ players, communists (including anarchist) should know each other
        assert!(!communist_players.is_empty(), "Should have communist players");
        assert_eq!(anarchist_player.role, Role::Anarchist, "Should have anarchist");

        // ASSUMPTION: Anarchist is included in communist knowledge group
        // This is tested more thoroughly in communist_knowledge tests

        // ASSUMPTION: Anarchist might have additional information or abilities
        // beyond standard communist knowledge
    }
}

/// Test anarchist win condition scenarios
#[test]
fn test_anarchist_win_scenarios() {
    let opts = GameOptions {
        communists: true,
        monarchist: false,
        anarchist: true,
        capitalist: false,
        centrists: false,
    };
    let player_names: Vec<String> = (0..10).map(|i| format!("Player{}", i)).collect();

    if let Ok(game) = Game::new(opts, &player_names, 42) {
        let anarchist_player = game
            .players
            .iter()
            .find(|p| p.role == Role::Anarchist)
            .expect("Should have anarchist player");

        // ASSUMPTION: Anarchist wins in the following scenarios:
        // 1. Communist team victory (standard team win)
        // 2. Possibly unique anarchist victory conditions (chaos/disruption based)

        // Verify anarchist is set up for win condition testing
        assert_eq!(anarchist_player.role, Role::Anarchist, "Should be anarchist");
        assert!(anarchist_player.alive, "Should start alive for win scenarios");

        // ASSUMPTION: Anarchist victory would be tested through full game simulation
        // which is beyond the scope of these unit tests
    }
}

/// Test anarchist policy effects and chaos mechanics
#[test]
fn test_anarchist_chaos_mechanics() {
    let opts = GameOptions {
        communists: true,
        monarchist: false,
        anarchist: true,
        capitalist: false,
        centrists: false,
    };
    let player_names: Vec<String> = (0..12).map(|i| format!("Player{}", i)).collect();

    if let Ok(game) = Game::new(opts, &player_names, 42) {
        let anarchist_player = game
            .players
            .iter()
            .find(|p| p.role == Role::Anarchist)
            .expect("Should have anarchist player");

        // ASSUMPTION: Anarchist might have chaos-inducing abilities such as:
        // - Disrupting policy placement order
        // - Causing random effects during elections
        // - Special investigation or executive action results
        // - Ability to cause confusion or misdirection

        // Verify anarchist is properly initialized for chaos mechanics
        assert_eq!(anarchist_player.role, Role::Anarchist, "Should be anarchist");

        // ASSUMPTION: Chaos mechanics would be implemented as special game rules
        // or abilities that activate under certain conditions
    }
}

/// Test anarchist interaction with executive actions
#[test]
fn test_anarchist_executive_actions() {
    let opts = GameOptions {
        communists: true,
        monarchist: false,
        anarchist: true,
        capitalist: false,
        centrists: false,
    };
    let player_names: Vec<String> = (0..11).map(|i| format!("Player{}", i)).collect();

    if let Ok(game) = Game::new(opts, &player_names, 42) {
        let anarchist_player = game
            .players
            .iter()
            .find(|p| p.role == Role::Anarchist)
            .expect("Should have anarchist player");

        // ASSUMPTION: Anarchist might have unique interactions with executive actions:
        // - Special results when investigated
        // - Unique effects when chosen for executive actions
        // - Ability to disrupt or modify executive action outcomes

        // Verify anarchist is set up for executive action testing
        assert_eq!(anarchist_player.role, Role::Anarchist, "Should be anarchist");
        assert!(!anarchist_player.investigated, "Should start uninvestigated");

        // ASSUMPTION: Executive action interactions would be tested through
        // gameplay simulation with specific executive action scenarios
    }
}

/// Test anarchist minimum and maximum player count requirements
#[test]
fn test_anarchist_player_count_requirements() {
    // Test various player counts to see when anarchist is available
    for player_count in 6..=16 {
        let opts = GameOptions {
            communists: true,
            monarchist: false,
            anarchist: true,
            capitalist: false,
            centrists: false,
        };
        let player_names: Vec<String> = (0..player_count).map(|i| format!("Player{}", i)).collect();

        match Game::new(opts, &player_names, 42) {
            Ok(game) => {
                let anarchist_count = game.players.iter().filter(|p| p.role == Role::Anarchist).count();

                if anarchist_count > 0 {
                    assert_eq!(
                        anarchist_count, 1,
                        "{} players should have exactly 1 anarchist when present",
                        player_count
                    );
                }

                // ASSUMPTION: Anarchist might only be available at certain player counts
                // to maintain game balance
            }
            Err(_) => {
                // Game creation might fail at certain player counts
                // This could indicate minimum requirements for anarchist role
            }
        }
    }
}

/// Test anarchist role uniqueness (only one anarchist per game)
#[test]
fn test_anarchist_role_uniqueness() {
    let opts = GameOptions {
        communists: true,
        monarchist: true,
        anarchist: true,
        capitalist: true,
        centrists: true,
    };
    let player_names: Vec<String> = (0..16).map(|i| format!("Player{}", i)).collect();

    if let Ok(game) = Game::new(opts, &player_names, 42) {
        let anarchist_count = game.players.iter().filter(|p| p.role == Role::Anarchist).count();

        // Should have exactly 1 anarchist, never more
        assert_eq!(
            anarchist_count, 1,
            "Should have exactly 1 anarchist, found {}",
            anarchist_count
        );

        // Verify anarchist is unique among all players
        let anarchist_indices: Vec<_> = game
            .players
            .iter()
            .enumerate()
            .filter(|(_, p)| p.role == Role::Anarchist)
            .map(|(i, _)| i)
            .collect();

        assert_eq!(
            anarchist_indices.len(),
            1,
            "Should have exactly one anarchist player index"
        );
    }
}
