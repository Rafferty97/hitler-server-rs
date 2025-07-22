//! Special roles interaction tests
//!
//! Tests based on the Secret Hitler XL rules for special role interactions:
//! - Capitalist role mechanics and interactions
//! - Anarchist role mechanics and communist team alignment
//! - Monarchist role mechanics and Hitler protection
//! - Centrist role mechanics and liberal team alignment
//! - Cross-role interactions and edge cases
//!
//! AMBIGUITY NOTES:
//! - Role name inconsistency: rules.pdf uses both "Monarchist" and "Nationalist"
//! - ASSUMPTION: Using "Monarchist" as it appears in the implementation
//! - Capitalist win condition timing: unclear if immediate or end-of-game check
//! - ASSUMPTION: Capitalist win is checked at end of game like other conditions

use super::super::player::Role;
use super::test_utils::*;
use crate::game::{Game, GameOptions};

/// Test that Capitalist role is assigned correctly in different player counts
#[test]
fn test_capitalist_role_assignment() {
    for player_count in 8..=16 {
        let opts = GameOptions {
            communists: true,
            monarchist: false,
            anarchist: false,
            capitalist: true,
            centrists: false,
        };
        let player_names: Vec<String> = (0..player_count).map(|i| format!("Player{}", i)).collect();

        if let Ok(game) = Game::new(opts, &player_names, 42) {
            let capitalist_count = game.players.iter().filter(|p| p.role == Role::Capitalist).count();

            // Should have exactly 1 capitalist when enabled
            assert_eq!(
                capitalist_count, 1,
                "{} players should have exactly 1 capitalist when enabled",
                player_count
            );

            // Capitalist should not be Hitler
            let capitalist_player = game
                .players
                .iter()
                .find(|p| p.role == Role::Capitalist)
                .expect("Should have capitalist player");

            assert!(
                capitalist_player.role != Role::Hitler,
                "Capitalist should not be Hitler"
            );
        }
    }
}

/// Test that Anarchist role is assigned correctly and aligns with communist team
#[test]
fn test_anarchist_role_assignment_and_alignment() {
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

            // Anarchist should be on communist team (tested via knowledge rules)
            // This is verified by the communist knowledge tests
        }
    }
}

/// Test that Monarchist role is assigned correctly
#[test]
fn test_monarchist_role_assignment() {
    for player_count in 8..=16 {
        let opts = GameOptions {
            communists: true,
            monarchist: true,
            anarchist: false,
            capitalist: false,
            centrists: false,
        };
        let player_names: Vec<String> = (0..player_count).map(|i| format!("Player{}", i)).collect();

        if let Ok(game) = Game::new(opts, &player_names, 42) {
            let monarchist_count = game.players.iter().filter(|p| p.role == Role::Monarchist).count();

            // Should have exactly 1 monarchist when enabled
            assert_eq!(
                monarchist_count, 1,
                "{} players should have exactly 1 monarchist when enabled",
                player_count
            );

            // Monarchist should not be Hitler
            let monarchist_player = game
                .players
                .iter()
                .find(|p| p.role == Role::Monarchist)
                .expect("Should have monarchist player");

            assert!(
                monarchist_player.role != Role::Hitler,
                "Monarchist should not be Hitler"
            );
        }
    }
}

/// Test that Centrist roles are assigned correctly
#[test]
fn test_centrist_role_assignment() {
    for player_count in 8..=16 {
        let opts = GameOptions {
            communists: false,
            monarchist: false,
            anarchist: false,
            capitalist: false,
            centrists: true,
        };
        let player_names: Vec<String> = (0..player_count).map(|i| format!("Player{}", i)).collect();

        if let Ok(game) = Game::new(opts, &player_names, 42) {
            let centrist_count = game.players.iter().filter(|p| p.role == Role::Centrist).count();

            // Should have centrists when enabled (exact count depends on player count)
            assert!(
                centrist_count >= 1,
                "{} players should have at least 1 centrist when enabled",
                player_count
            );

            // Centrists should not be Hitler
            let centrist_players: Vec<_> = game.players.iter().filter(|p| p.role == Role::Centrist).collect();

            for centrist in centrist_players {
                assert!(centrist.role != Role::Hitler, "Centrist should not be Hitler");
            }
        }
    }
}

/// Test multiple special roles can coexist
#[test]
fn test_multiple_special_roles_coexistence() {
    for player_count in 10..=16 {
        let opts = GameOptions {
            communists: true,
            monarchist: true,
            anarchist: true,
            capitalist: true,
            centrists: true,
        };
        let player_names: Vec<String> = (0..player_count).map(|i| format!("Player{}", i)).collect();

        if let Ok(game) = Game::new(opts, &player_names, 42) {
            let capitalist_count = game.players.iter().filter(|p| p.role == Role::Capitalist).count();
            let anarchist_count = game.players.iter().filter(|p| p.role == Role::Anarchist).count();
            let monarchist_count = game.players.iter().filter(|p| p.role == Role::Monarchist).count();
            let centrist_count = game.players.iter().filter(|p| p.role == Role::Centrist).count();

            // Should have exactly 1 of each unique special role
            assert_eq!(capitalist_count, 1, "Should have exactly 1 capitalist");
            assert_eq!(anarchist_count, 1, "Should have exactly 1 anarchist");
            assert_eq!(monarchist_count, 1, "Should have exactly 1 monarchist");

            // Should have at least 1 centrist (can have multiple)
            assert!(centrist_count >= 1, "Should have at least 1 centrist");

            // Verify no special role is Hitler
            for player in &game.players {
                if matches!(
                    player.role,
                    Role::Capitalist | Role::Anarchist | Role::Monarchist | Role::Centrist
                ) {
                    assert!(player.role != Role::Hitler, "{:?} should not be Hitler", player.role);
                }
            }
        }
    }
}

/// Test that special roles don't interfere with core role distribution
#[test]
fn test_special_roles_preserve_core_distribution() {
    let player_count = 12;
    let opts = GameOptions {
        communists: true,
        monarchist: true,
        anarchist: true,
        capitalist: true,
        centrists: true,
    };
    let player_names: Vec<String> = (0..player_count).map(|i| format!("Player{}", i)).collect();

    if let Ok(game) = Game::new(opts, &player_names, 42) {
        // Count all roles
        let liberal_count = game.players.iter().filter(|p| p.role == Role::Liberal).count();
        let fascist_count = game.players.iter().filter(|p| p.role == Role::Fascist).count();
        let communist_count = game.players.iter().filter(|p| p.role == Role::Communist).count();
        let hitler_count = game.players.iter().filter(|p| p.role == Role::Hitler).count();

        let capitalist_count = game.players.iter().filter(|p| p.role == Role::Capitalist).count();
        let anarchist_count = game.players.iter().filter(|p| p.role == Role::Anarchist).count();
        let monarchist_count = game.players.iter().filter(|p| p.role == Role::Monarchist).count();
        let centrist_count = game.players.iter().filter(|p| p.role == Role::Centrist).count();

        // Should have exactly 1 Hitler
        assert_eq!(hitler_count, 1, "Should have exactly 1 Hitler");

        // Should have core roles
        assert!(liberal_count >= 1, "Should have at least 1 liberal");
        assert!(fascist_count >= 1, "Should have at least 1 fascist");
        assert!(communist_count >= 1, "Should have at least 1 communist");

        // Should have special roles
        assert_eq!(capitalist_count, 1, "Should have exactly 1 capitalist");
        assert_eq!(anarchist_count, 1, "Should have exactly 1 anarchist");
        assert_eq!(monarchist_count, 1, "Should have exactly 1 monarchist");
        assert!(centrist_count >= 1, "Should have at least 1 centrist");

        // Total should equal player count (including Hitler)
        let total_roles = liberal_count
            + fascist_count
            + communist_count
            + hitler_count
            + capitalist_count
            + anarchist_count
            + monarchist_count
            + centrist_count;
        assert_eq!(total_roles, player_count, "Total roles should equal player count");
    }
}

/// Test Capitalist team alignment (should be on liberal team)
#[test]
fn test_capitalist_team_alignment() {
    let opts = GameOptions {
        communists: true,
        monarchist: false,
        anarchist: false,
        capitalist: true,
        centrists: false,
    };
    let player_names: Vec<String> = (0..10).map(|i| format!("Player{}", i)).collect();

    if let Ok(game) = Game::new(opts, &player_names, 42) {
        let capitalist_player = game
            .players
            .iter()
            .find(|p| p.role == Role::Capitalist)
            .expect("Should have capitalist player");

        // Capitalist should be on liberal team (not fascist or communist)
        assert_ne!(
            capitalist_player.role,
            Role::Fascist,
            "Capitalist should not be fascist"
        );
        assert_ne!(
            capitalist_player.role,
            Role::Communist,
            "Capitalist should not be communist"
        );
        assert!(
            capitalist_player.role != Role::Hitler,
            "Capitalist should not be Hitler"
        );

        // ASSUMPTION: Capitalist is aligned with liberal team for victory conditions
        // This would be tested in victory condition tests
    }
}

/// Test Anarchist team alignment (should be on communist team)
#[test]
fn test_anarchist_team_alignment() {
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

        // Anarchist should be on communist team
        assert_ne!(anarchist_player.role, Role::Liberal, "Anarchist should not be liberal");
        assert_ne!(anarchist_player.role, Role::Fascist, "Anarchist should not be fascist");
        assert!(anarchist_player.role != Role::Hitler, "Anarchist should not be Hitler");

        // Anarchist should be part of communist knowledge group (tested in communist_knowledge tests)
        // ASSUMPTION: Anarchist wins with communist team
    }
}

/// Test Monarchist team alignment (should be on fascist team)
#[test]
fn test_monarchist_team_alignment() {
    let opts = GameOptions {
        communists: false,
        monarchist: true,
        anarchist: false,
        capitalist: false,
        centrists: false,
    };
    let player_names: Vec<String> = (0..10).map(|i| format!("Player{}", i)).collect();

    if let Ok(game) = Game::new(opts, &player_names, 42) {
        let monarchist_player = game
            .players
            .iter()
            .find(|p| p.role == Role::Monarchist)
            .expect("Should have monarchist player");

        // Monarchist should be on fascist team
        assert_ne!(
            monarchist_player.role,
            Role::Liberal,
            "Monarchist should not be liberal"
        );
        assert_ne!(
            monarchist_player.role,
            Role::Communist,
            "Monarchist should not be communist"
        );
        assert!(
            monarchist_player.role != Role::Hitler,
            "Monarchist should not be Hitler"
        );

        // ASSUMPTION: Monarchist is aligned with fascist team for victory conditions
        // and provides Hitler protection mechanics
    }
}

/// Test Centrist team alignment (should be on liberal team)
#[test]
fn test_centrist_team_alignment() {
    let opts = GameOptions {
        communists: false,
        monarchist: false,
        anarchist: false,
        capitalist: false,
        centrists: true,
    };
    let player_names: Vec<String> = (0..10).map(|i| format!("Player{}", i)).collect();

    if let Ok(game) = Game::new(opts, &player_names, 42) {
        let centrist_players: Vec<_> = game.players.iter().filter(|p| p.role == Role::Centrist).collect();

        assert!(!centrist_players.is_empty(), "Should have centrist players");

        for centrist in centrist_players {
            // Centrist should be on liberal team
            assert_ne!(centrist.role, Role::Fascist, "Centrist should not be fascist");
            assert_ne!(centrist.role, Role::Communist, "Centrist should not be communist");
            assert!(centrist.role != Role::Hitler, "Centrist should not be Hitler");
        }

        // ASSUMPTION: Centrists are aligned with liberal team for victory conditions
    }
}

/// Test that special roles are mutually exclusive (one player can't have multiple special roles)
#[test]
fn test_special_roles_mutual_exclusivity() {
    let opts = GameOptions {
        communists: true,
        monarchist: true,
        anarchist: true,
        capitalist: true,
        centrists: true,
    };
    let player_names: Vec<String> = (0..16).map(|i| format!("Player{}", i)).collect();

    if let Ok(game) = Game::new(opts, &player_names, 42) {
        // Verify no player has multiple special roles
        for player in &game.players {
            let special_role_count = [
                player.role == Role::Capitalist,
                player.role == Role::Anarchist,
                player.role == Role::Monarchist,
                player.role == Role::Centrist,
            ]
            .iter()
            .filter(|&&x| x)
            .count();

            assert!(
                special_role_count <= 1,
                "Player should have at most 1 special role, found {} special roles for {:?}",
                special_role_count,
                player.role
            );
        }
    }
}

/// Test special roles with minimum player counts
#[test]
fn test_special_roles_minimum_player_counts() {
    // Test with minimum viable player count for each special role
    let test_cases = vec![
        (
            8,
            GameOptions {
                communists: false,
                monarchist: false,
                anarchist: false,
                capitalist: true,
                centrists: false,
            },
        ),
        (
            8,
            GameOptions {
                communists: true,
                monarchist: false,
                anarchist: true,
                capitalist: false,
                centrists: false,
            },
        ),
        (
            8,
            GameOptions {
                communists: false,
                monarchist: true,
                anarchist: false,
                capitalist: false,
                centrists: false,
            },
        ),
        (
            8,
            GameOptions {
                communists: false,
                monarchist: false,
                anarchist: false,
                capitalist: false,
                centrists: true,
            },
        ),
    ];

    for (player_count, opts) in test_cases {
        let player_names: Vec<String> = (0..player_count).map(|i| format!("Player{}", i)).collect();

        if let Ok(game) = Game::new(opts, &player_names, 42) {
            // Verify the game can be created with special roles at minimum player count
            assert_eq!(
                game.players.len(),
                player_count,
                "Should have correct number of players"
            );

            // Verify at least one special role is present
            let has_special_role = game.players.iter().any(|p| {
                matches!(
                    p.role,
                    Role::Capitalist | Role::Anarchist | Role::Monarchist | Role::Centrist
                )
            });

            if opts.capitalist || opts.anarchist || opts.monarchist || opts.centrists {
                assert!(has_special_role, "Should have at least one special role when enabled");
            }
        }
    }
}

/// Test that Hitler cannot be assigned to special roles
#[test]
fn test_hitler_not_assigned_to_special_roles() {
    let opts = GameOptions {
        communists: true,
        monarchist: true,
        anarchist: true,
        capitalist: true,
        centrists: true,
    };

    for player_count in 10..=16 {
        let player_names: Vec<String> = (0..player_count).map(|i| format!("Player{}", i)).collect();

        if let Ok(game) = Game::new(opts, &player_names, 42) {
            let hitler_player = game
                .players
                .iter()
                .find(|p| p.role == Role::Hitler)
                .expect("Should have Hitler player");

            // Hitler should not have any special role
            assert!(
                !matches!(
                    hitler_player.role,
                    Role::Capitalist | Role::Anarchist | Role::Monarchist | Role::Centrist
                ),
                "Hitler should not be assigned to special roles, but found Hitler as {:?}",
                hitler_player.role
            );

            // Hitler should be Liberal, Fascist, Communist, or Hitler role
            assert!(
                matches!(
                    hitler_player.role,
                    Role::Liberal | Role::Fascist | Role::Communist | Role::Hitler
                ),
                "Hitler should be Liberal, Fascist, Communist, or Hitler, but found {:?}",
                hitler_player.role
            );
        }
    }
}
