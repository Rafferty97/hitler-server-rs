use thiserror::Error;

/// The result of attempting to perform an invalid operation on a [Game] or [Session].
#[derive(Error, Debug)]
pub enum GameError {
    #[error("invalid combination of game options")]
    InvalidGameOptions,
    #[error("game does not exist")]
    GameNotFound,
    #[error("too few players in the game")]
    TooFewPlayers,
    #[error("too many players in the game")]
    TooManyPlayers,
    #[error("no player exists with the given name")]
    PlayerNotFound,
    #[error("cannot join a game in progress")]
    CannotJoinStartedGame,
    #[error("this player cannot be chosen for this action")]
    InvalidPlayerChoice,
    #[error("invalid player index")]
    InvalidPlayerIndex,
    #[error("this action cannot be performed during this phase of the game")]
    InvalidAction,
    #[error("an invalid card was chosen")]
    InvalidCard,
}
