use super::{party::Party, GameOptions, MAX_PLAYERS};
use crate::error::GameError;
use rand::prelude::SliceRandom;
use serde::{Deserialize, Serialize};
use std::iter::repeat;

/// A game player.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Player {
    pub name: String,
    pub role: Role,
    pub alive: bool,
    pub not_hitler: bool,
    pub investigated: bool,
    pub others: [InvestigationResult; MAX_PLAYERS],
}

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub enum Role {
    Liberal,
    Fascist,
    Communist,
    Hitler,
    Monarchist,
    Anarchist,
    Capitalist,
    Centrist,
}

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub enum InvestigationResult {
    Unknown,
    Party(Party),
    Role(Role),
}

impl ToString for Role {
    fn to_string(&self) -> String {
        match self {
            Role::Liberal => "Liberal",
            Role::Fascist => "Fascist",
            Role::Communist => "Communist",
            Role::Hitler => "Hitler",
            Role::Monarchist => "Monarchist",
            Role::Anarchist => "Anarchist",
            Role::Capitalist => "Capitalist",
            Role::Centrist => "Centrist",
        }
        .to_string()
    }
}

impl Player {
    pub fn new(name: String, role: Role) -> Self {
        Self {
            name,
            role,
            others: [InvestigationResult::Unknown; MAX_PLAYERS],
            alive: true,
            not_hitler: false,
            investigated: false,
        }
    }

    pub fn party(&self) -> Party {
        match self.role {
            Role::Liberal => Party::Liberal,
            Role::Fascist => Party::Fascist,
            Role::Communist => Party::Communist,
            Role::Hitler => Party::Fascist,
            Role::Monarchist => Party::Fascist,
            Role::Anarchist => Party::Communist,
            Role::Capitalist => Party::Liberal,
            Role::Centrist => Party::Liberal,
        }
    }
}

pub fn assign_roles(
    num_players: usize,
    opts: &GameOptions,
    rng: &mut impl rand::Rng,
) -> Result<Vec<Role>, GameError> {
    let mut roles = vec![];

    // Determine the total number of fascists and communists to include
    let mut fascists = match (opts.communists, num_players) {
        (false, ..=4) => return Err(GameError::TooFewPlayers),
        (false, 5..=6) => 2,
        (false, 7..=8) => 3,
        (false, 9..=10) => 4,
        (true, ..=5) => return Err(GameError::TooFewPlayers),
        (true, 6..=7) => 2,
        (true, 8..=10) => 3,
        (true, 11..=14) => 4,
        (true, 15..=16) => 5,
        _ => return Err(GameError::TooManyPlayers),
    };

    let mut communists = match (opts.communists, num_players) {
        (false, _) => 0,
        (true, ..=5) => return Err(GameError::TooFewPlayers),
        (true, 6..=8) => 1,
        (true, 9..=12) => 2,
        (true, 13..=15) => 3,
        (true, 16) => 4,
        _ => return Err(GameError::TooManyPlayers),
    };

    // Add the fascist players
    roles.push(Role::Hitler);
    fascists -= 1;

    if opts.monarchist {
        roles.push(Role::Monarchist);
        fascists -= 1;
    }

    roles.extend(repeat(Role::Fascist).take(fascists));

    // Add the communist players
    if communists > 0 && opts.anarchist {
        roles.push(Role::Anarchist);
        communists -= 1;
    }

    roles.extend(repeat(Role::Communist).take(communists));

    // Add the liberal players
    if opts.capitalist {
        roles.push(Role::Capitalist);
    }

    if opts.centrists {
        roles.push(Role::Centrist);
        roles.push(Role::Centrist);
    }

    let liberals = num_players - roles.len();
    roles.extend(repeat(Role::Liberal).take(liberals));

    // Shuffle the roles and return
    roles.shuffle(rng);
    Ok(roles)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::game::player::Role;
    use crate::game::GameOptions;

    #[test]
    fn role_assignment_10players() {
        // 2+H, 5, 2
        let opts = GameOptions {
            communists: true,
            anarchist: true,
            capitalist: true,
            centrists: true,
            monarchist: false,
        };
        let roles = assign_roles(10, &opts, &mut rand::thread_rng()).unwrap();
        assert_eq!(roles.iter().filter(|r| **r == Role::Hitler).count(), 1);
        assert_eq!(roles.iter().filter(|r| **r == Role::Monarchist).count(), 0);
        assert_eq!(roles.iter().filter(|r| **r == Role::Fascist).count(), 2);
        assert_eq!(roles.iter().filter(|r| **r == Role::Capitalist).count(), 1);
        assert_eq!(roles.iter().filter(|r| **r == Role::Centrist).count(), 2);
        assert_eq!(roles.iter().filter(|r| **r == Role::Liberal).count(), 2);
        assert_eq!(roles.iter().filter(|r| **r == Role::Anarchist).count(), 1);
        assert_eq!(roles.iter().filter(|r| **r == Role::Communist).count(), 1);
    }
}
