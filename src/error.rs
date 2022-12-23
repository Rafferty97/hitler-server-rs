use thiserror::Error;

/// The result of attempting to perform an invalid operation on a [Game] or [Session].
#[derive(Error, Debug)]
pub enum GameError {
    #[error("game does not exist")]
    GameNotFound,
    #[error("cannot have more than 10 players in a game")]
    TooManyPlayers,
    #[error("cannot join a game that has already started")]
    CannotJoinStartedGame,
    #[error("no player exists with the given name")]
    PlayerNotFound,
    #[error("this player cannot be chosen for this action")]
    InvalidPlayerChoice,
    #[error("invalid player index")]
    InvalidPlayerIndex,
    #[error("this action cannot be performed during this phase of the game")]
    InvalidAction,
    #[error("an invalid card was chosen")]
    InvalidCard,
}