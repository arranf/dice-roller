use dice_command_parser::error::ParserError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DiceError {
    #[error("Error parsing input: {0}")]
    ParseError(#[from] ParserError),
    #[error("An unknown error occurred")]
    Unknown,
}
