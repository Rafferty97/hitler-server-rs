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
    pub others: [InvestigationResult; MAX_PLAYERS],
    pub alive: bool,
    pub not_hitler: bool,
    pub investigated: bool,
    pub tried_to_radicalise: bool,
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

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub enum InvestigationResult {
    Unknown,
    Party(Party),
    Role(Role),
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
            tried_to_radicalise: false,
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

    pub fn radicalise(&mut self) -> bool {
        self.tried_to_radicalise = true;
        if matches!(self.role, Role::Liberal | Role::Centrist) {
            self.role = Role::Communist;
            true
        } else {
            false
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PlayerDistribution {
    pub num_players: usize,
    pub liberals: usize,
    pub fascists: usize,
    pub communists: usize,
    pub hitler: bool,
    pub monarchist: bool,
    pub anarchist: bool,
    pub capitalist: bool,
    pub centrists: bool,
}

impl PlayerDistribution {
    pub fn new(opts: &GameOptions, num_players: usize) -> Result<Self, GameError> {
        let mut fascists: isize;
        let mut communists: isize;
        let mut liberals: isize;

        // Calculate the number of players in each party
        if opts.communists {
            fascists = match num_players {
                ..=5 => return Err(GameError::TooFewPlayers),
                6..=7 => 2,
                8..=10 => 3,
                11..=14 => 4,
                15..=16 => 5,
                _ => return Err(GameError::TooManyPlayers),
            };
            communists = match num_players {
                ..=5 => return Err(GameError::TooFewPlayers),
                6..=8 => 1,
                9..=12 => 2,
                13..=15 => 3,
                16 => 4,
                _ => return Err(GameError::TooManyPlayers),
            };
        } else {
            fascists = match num_players {
                ..=4 => return Err(GameError::TooFewPlayers),
                5..=10 => (num_players as isize - 1) / 2,
                _ => return Err(GameError::TooManyPlayers),
            };
            communists = 0;
        }
        liberals = num_players as isize - (fascists + communists);

        // Subtract away the special roles
        let hitler = true;
        let GameOptions {
            monarchist, anarchist, capitalist, centrists, ..
        } = *opts;

        fascists -= hitler as isize;
        fascists -= monarchist as isize;
        communists -= anarchist as isize;
        liberals -= capitalist as isize;
        liberals -= 2 * (centrists as isize);

        // Ensure enough "ordinary" players remain
        let min_communists = opts.communists as isize;
        if fascists < 1 || communists < min_communists || liberals < 0 {
            return Err(GameError::TooFewPlayers);
        }

        // Return the result
        Ok(Self {
            num_players,
            liberals: liberals as usize,
            fascists: fascists as usize,
            communists: communists as usize,
            hitler,
            monarchist,
            anarchist,
            capitalist,
            centrists,
        })
    }
}

pub fn assign_roles(distr: PlayerDistribution, rng: &mut impl rand::Rng) -> Vec<Role> {
    let mut roles = Vec::with_capacity(distr.num_players);

    roles.extend(repeat(Role::Fascist).take(distr.fascists));
    roles.extend(repeat(Role::Communist).take(distr.communists));
    roles.extend(repeat(Role::Liberal).take(distr.liberals));

    if distr.hitler {
        roles.push(Role::Hitler);
    }
    if distr.monarchist {
        roles.push(Role::Monarchist);
    }
    if distr.anarchist {
        roles.push(Role::Anarchist);
    }
    if distr.capitalist {
        roles.push(Role::Capitalist);
    }
    if distr.centrists {
        roles.push(Role::Centrist);
        roles.push(Role::Centrist);
    }

    assert_eq!(roles.len(), distr.num_players);

    roles.shuffle(rng);
    roles
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
        let distr = PlayerDistribution::new(&opts, 10).unwrap();
        println!("{:?}", &distr);
        let roles = assign_roles(distr, &mut rand::thread_rng());
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
