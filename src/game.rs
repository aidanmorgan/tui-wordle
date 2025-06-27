use crate::dictionary::{get_dictionaries, Dictionary};
use std::cmp::PartialEq;
use std::fmt::{Debug};
use std::sync::Arc;

#[derive(Debug, thiserror::Error)]
pub enum GameError {
    #[error("Error accessing dictionary")]
    DictionaryError,

    #[error("No active game")]
    NoActiveGame,

    #[error("No active guess")]
    NoActiveGuess,

    #[error("Cannot delete from an empty guess")]
    EmptyGuess,

    #[error("Cannot add to a full guess")]
    FullGuess,

    #[error("Guess is incomplete")]
    IncompleteGuess,

    #[error("Internal error: {0}")]
    InternalError(String),
}

#[derive(Hash, Eq, PartialEq, Clone, Debug, Copy)]
pub enum LetterResult {
    Empty,
    Absent,
    Present,
    Correct,
}

/// Game configuration options
#[derive(Debug, Clone)]
pub struct GameOptions {
    /// Length of words in the game
    pub word_length: u16,
    /// Maximum number of guesses allowed
    pub max_guesses: u16,
    /// Dictionary used for the game
    pub dictionary: Arc<Dictionary>,
}

impl Default for GameOptions {
    fn default() -> Self {
        let dictionaries = get_dictionaries();

        // Find the default Wordle dictionary or use the first available dictionary
        let default_dictionary = dictionaries
            .iter()
            .find(|x| x.name == "Wordle" && x.length == 5)
            .or_else(|| dictionaries.first())
            .expect("No dictionaries available");

        Self {
            word_length: default_dictionary.length as u16,
            max_guesses: 6,
            dictionary: Arc::clone(default_dictionary),
        }
    }
}

impl GameOptions {
    /// Gets a random word from the current dictionary
    pub fn random_word(&self) -> Result<String, GameError> {
        self.dictionary
            .random_word()
            .map_err(|_e| GameError::DictionaryError)
    }

    /// Sets the dictionary to use for the game
    ///
    /// # Arguments
    /// * `name` - The name of the dictionary
    /// * `length` - The length of words in the dictionary
    ///
    /// # Returns
    /// * `Ok(())` if the dictionary was set successfully
    /// * `Err(GameError::DictionaryError)` if the dictionary was not found
    pub fn set_dictionary(&mut self, name: &str, length: u8) -> Result<(), GameError> {
        let dictionaries = get_dictionaries();

        let dictionary = dictionaries
            .iter()
            .find(|x| x.name == name && x.length == length)
            .ok_or_else(|| GameError::DictionaryError)?;

        self.dictionary = Arc::clone(dictionary);
        self.word_length = length as u16;

        Ok(())
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Copy)]
enum GuessState {
    Active,
    Complete,
    Pending,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Guess {
    max_length: u16,
    letters: Vec<char>,
    result: Option<Vec<LetterResult>>,
    state: GuessState,
}

impl Guess {
    fn new(max_length: u16) -> Self {
        Self {
            max_length,
            letters: Vec::new(),
            result: None,
            state: GuessState::Pending,
        }
    }

    fn make_vec(word_length: u16, max_guesses: u16) -> Vec<Guess> {
        (0..max_guesses)
            .map(|i| {
                let mut g = Guess::new(word_length);
                if i == 0 {
                    g.state = GuessState::Active;
                }
                g
            })
            .collect()
    }

    fn add_letter(&mut self, c: char) -> Result<(), GameError> {
        if self.remaining_letters() <= 0 {
            return Err(GameError::FullGuess);
        }

        self.letters.push(c);
        Ok(())
    }

    fn delete_letter(&mut self) -> Result<(), GameError> {
        if self.remaining_letters() == self.max_length {
            return Err(GameError::EmptyGuess);
        }

        self.letters.pop();
        Ok(())
    }

    fn remaining_letters(&self) -> u16 {
        self.max_length - (self.letters.len() as u16)
    }

    pub fn as_chars(&self) -> Vec<char> {
        self.letters.clone()
    }

    fn complete_guess(&mut self, result: &Vec<LetterResult>) {
        self.result = Some(result.clone());
        self.state = GuessState::Complete;
    }

    pub fn values(&self) -> Vec<(Option<char>, Option<LetterResult>)> {
        if let Some(results) = &self.result {
            self.letters
                .iter()
                .zip(results.iter())
                .map(|(c, result)| (Some(*c), Some(*result)))
                .collect()
        } else {
            let mut result = vec![(Some(' '), Some(LetterResult::Empty)); self.max_length as usize];

            for (i, c) in self.letters.iter().enumerate() {
                result[i] = (Some(c.to_ascii_uppercase()), Some(LetterResult::Empty));
            }

            result
        }
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub enum GameState {
    Active,
    Won,
    Lost,
}

#[derive(Debug)]
pub struct GameData {
    pub game_state: GameState,
    pub answer: String,
    game_options: GameOptions,
    pub guesses: Vec<Guess>,
}

impl GameData {
    /// Creates a new game with the given options
    ///
    /// # Panics
    /// Panics if a random word cannot be retrieved from the dictionary.
    /// This is a design choice to fail fast if the dictionary is not available,
    /// as the game cannot function without a word to guess.
    pub fn new(opts: &GameOptions) -> Self {
        let word = opts
            .random_word()
            .unwrap_or_else(|e| {
                eprintln!("Failed to get random word: {}", e);
                panic!("Cannot start game without a word to guess")
            });

        Self {
            game_state: GameState::Active,
            game_options: opts.clone(),
            answer: word,
            guesses: Guess::make_vec(opts.word_length, opts.max_guesses),
        }
    }

    fn active_guess(&mut self) -> Option<(u16, &mut Guess)> {
        let idx = self
            .guesses
            .iter()
            .position(|x| x.state == GuessState::Active);

        if let Some(x) = idx {
            Some((x as u16, &mut self.guesses[x]))
        } else {
            None
        }
    }

    fn is_active(&self) -> bool {
        matches!(self.game_state, GameState::Active)
    }

    pub fn add_letter(&mut self, val: char) -> Result<(), GameError> {
        if !self.is_active() {
            return Err(GameError::NoActiveGame);
        }

        let guess = self.active_guess().ok_or(GameError::NoActiveGuess)?;
        guess.1.add_letter(val.to_ascii_uppercase())
    }

    pub fn delete_letter(&mut self) -> Result<(), GameError> {
        if !self.is_active() {
            return Err(GameError::NoActiveGame);
        }

        let guess = self.active_guess().ok_or(GameError::NoActiveGuess)?;
        guess.1.delete_letter()
    }

    /// Submits the current word and checks if it matches the answer
    ///
    /// # Returns
    /// * `Ok(GameState)` - The new state of the game
    /// * `Err(GameError)` - If there was an error submitting the word
    pub fn submit_word(&mut self) -> Result<GameState, GameError> {
        // Check if the game is active
        if self.game_state != GameState::Active {
            return Err(GameError::NoActiveGame);
        }

        // Find the active guess
        let active_guess = self.guesses
            .iter_mut()
            .enumerate()
            .find(|(_, g)| g.state == GuessState::Active)
            .ok_or_else(|| GameError::InternalError("No active guess found".to_string()))?;

        let (guess_idx, guess) = active_guess;
        let guess_idx = guess_idx as u16;

        // Check if the guess is complete
        if guess.remaining_letters() > 0 {
            return Err(GameError::IncompleteGuess);
        }

        // Get the guess characters before borrowing self again
        let guess_chars = guess.as_chars();

        // Process the guess
        let result = Self::check_guess(
            &self.answer,
            self.game_options.word_length,
            &guess_chars
        );

        guess.complete_guess(&result);

        // Update game state based on the result
        self.update_game_state(guess_idx, &result);

        Ok(self.game_state)
    }

    /// Checks a guess against the answer and returns the result
    fn check_guess(answer: &str, word_length: u16, guess_chars: &[char]) -> Vec<LetterResult> {
        let mut answer_chars: Vec<_> = answer.to_ascii_uppercase().chars().collect();
        let mut result = vec![LetterResult::Absent; word_length as usize];

        // First pass: check for correct letters
        for i in 0..guess_chars.len() {
            if guess_chars[i] == answer_chars[i].to_ascii_uppercase() {
                result[i] = LetterResult::Correct;
                // Mark this character as used
                answer_chars[i] = char::MIN;
            }
        }

        // Second pass: check for present letters
        for (i, &g) in guess_chars.iter().enumerate() {
            // Skip letters that are already marked as correct
            if result[i] == LetterResult::Correct {
                continue;
            }

            // Check if the letter is present elsewhere in the answer
            if let Some(pos) = answer_chars.iter().position(|&a| a.to_ascii_uppercase() == g) {
                result[i] = LetterResult::Present;
                // Mark this character as used
                answer_chars[pos] = char::MIN;
            }
        }

        result
    }

    /// Updates the game state based on the guess result
    fn update_game_state(&mut self, guess_idx: u16, result: &[LetterResult]) {
        if result.iter().all(|x| *x == LetterResult::Correct) {
            self.game_state = GameState::Won;
        } else if self.game_options.max_guesses - guess_idx - 1 > 0 {
            // Activate the next guess
            let next_guess = &mut self.guesses[(guess_idx + 1) as usize];
            next_guess.state = GuessState::Active;
        } else {
            self.game_state = GameState::Lost;
        }
    }
}
