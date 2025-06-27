use std::sync::Arc;
use thiserror::Error;
use crate::dictionary::{get_dictionaries, Dictionary};
use crate::game::{GameError, GameOptions};

/// Error type for options operations
#[derive(Debug, Error)]
pub enum OptionsError {
    #[error("Dictionary not found")]
    DictionaryNotFound,

    #[error("Game error: {0}")]
    GameError(#[from] GameError),
}

/// Represents the state of the options screen
#[derive(Debug)]
pub struct OptionData {
    /// Name of the selected dictionary
    pub(crate) dictionary_name: String,
    /// Length of words in the selected dictionary
    pub(crate) dictionary_length: u8,
    /// Maximum number of tries allowed
    pub(crate) max_tries: u16,
    /// Available dictionaries
    dictionaries: Vec<Arc<Dictionary>>,
}

impl OptionData {
    /// Creates a new OptionData with default values
    pub fn new() -> Self {
        Self {
            dictionary_name: String::from("Wordle"),
            dictionary_length: 5,
            max_tries: 6,
            dictionaries: get_dictionaries()
        }
    }

    /// Finds the current dictionary index
    fn find_dictionary_index(&self) -> Result<usize, OptionsError> {
        self.dictionaries
            .iter()
            .position(|dict| dict.name == self.dictionary_name && dict.length == self.dictionary_length)
            .ok_or(OptionsError::DictionaryNotFound)
    }

    /// Selects the next dictionary in the list
    pub fn next(&mut self) {
        if let Ok(idx) = self.find_dictionary_index() {
            let next = (idx + 1) % self.dictionaries.len();
            let dict = &self.dictionaries[next];
            self.dictionary_name = dict.name.clone();
            self.dictionary_length = dict.length;
        }
    }

    /// Selects the previous dictionary in the list
    pub fn previous(&mut self) {
        if let Ok(idx) = self.find_dictionary_index() {
            let prev = if idx == 0 {
                self.dictionaries.len() - 1
            } else {
                idx - 1
            };

            let dict = &self.dictionaries[prev];
            self.dictionary_name = dict.name.clone();
            self.dictionary_length = dict.length;
        }
    }

    /// Applies the current options to the game
    pub fn apply(&self, game_options: &mut GameOptions) -> Result<(), OptionsError> {
        game_options.set_dictionary(&self.dictionary_name, self.dictionary_length)?;
        game_options.max_guesses = self.max_tries;

        Ok(())
    }

    /// Increments the maximum number of tries (up to 10)
    pub fn increment_tries(&mut self) {
        self.max_tries = (self.max_tries + 1).min(10);
    }

    /// Decrements the maximum number of tries (down to 3)
    pub fn decrement_tries(&mut self) {
        self.max_tries = (self.max_tries - 1).max(3);
    }
}
