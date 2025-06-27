use crate::dictionary::{get_dictionaries, Dictionary};
use std::cmp::PartialEq;
use std::error;
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;

#[derive(Debug)]
pub enum GameError {
    FileLoadError,
    DictionaryError,
    NoActiveGame,

    EmptyGuess,
    FullGuess,
    IncompleteGuess,
    NoActiveGuess,
}
impl error::Error for GameError {}

impl Display for GameError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileLoadError => write!(f, "Failed to load game file"),
            Self::DictionaryError => write!(f, "Error accessing dictionary"),
            Self::NoActiveGame => write!(f, "No active game"),
            Self::EmptyGuess => write!(f, "Cannot delete from an empty guess"),
            Self::FullGuess => write!(f, "Cannot add to a full guess"),
            Self::IncompleteGuess => write!(f, "Guess is incomplete"),
            Self::NoActiveGuess => write!(f, "No active guess"),
        }
    }
}

#[derive(Hash, Eq, PartialEq, Clone, Debug, Copy)]
pub enum LetterResult {
    Empty,
    Absent,
    Present,
    Correct,
}

#[derive(Debug)]
pub struct GameOptions {
    pub word_length: u16,
    pub max_guesses: u16,
    pub dictionary: Arc<Dictionary>,
}

impl Clone for GameOptions {
    fn clone(&self) -> Self {
        GameOptions {
            dictionary: Arc::clone(&self.dictionary),
            max_guesses: self.max_guesses,
            word_length: self.word_length,
        }
    }
}

impl Default for GameOptions {
    fn default() -> Self {
        let dictionaries = get_dictionaries();

        let default_dictionary = dictionaries.iter().filter(|x| x.name == "Wordle").next();

        if let Some(x) = default_dictionary {
            Self {
                word_length: 5,
                max_guesses: 6,
                dictionary: Arc::clone(x),
            }
        } else {
            panic!("No default dictionary found")
        }
    }
}

impl GameOptions {
    pub fn random_word(&self) -> Result<String, GameError> {
        self.dictionary
            .random_word()
            .map_err(|_| GameError::DictionaryError)
    }

    fn set_dictionary(&mut self, name: &str, length: u8) -> Result<(), GameError> {
        let dictionaries = get_dictionaries();

        let dictionary = dictionaries
            .iter()
            .find(|x| x.name == name && x.length == length)
            .ok_or(GameError::DictionaryError)?;

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
                return g;
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
        if let Some(x) = &self.result {
            return self
                .letters
                .iter()
                .zip(x.iter())
                .map(|x| (Some(*x.0), Some(x.1.clone())))
                .collect();
        } else {
            let mut result = vec![(Some(' '), Some(LetterResult::Empty)); self.max_length as usize];

            for (i, x) in self.letters.iter().enumerate() {
                result[i] = (Some(x.to_ascii_uppercase()), Some(LetterResult::Empty));
            }

            return result;
        }
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub enum GameState {
    Active,
    Won,
    Lost,
}

pub struct GameData {
    pub game_state: GameState,
    game_options: GameOptions,
    pub(crate) answer: String,

    pub guesses: Vec<Guess>,
}

impl GameData {
    pub fn new(opts: &GameOptions) -> Self {
        let word = opts
            .random_word()
            .unwrap_or_else(|_| panic!("Failed to get random word"));

        let game_opts = opts.clone();

        Self {
            game_state: GameState::Active,
            game_options: game_opts,
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

    pub fn submit_word(&mut self) -> Result<GameState, GameError> {
        if self.game_state != GameState::Active {
            return Err(GameError::NoActiveGame);
        }

        let game_options = &self.game_options;
        let answer = &self.answer;
        let guesses = &mut self.guesses;

        let active_guess = guesses
            .iter_mut()
            .enumerate()
            .find(|x| x.1.state == GuessState::Active)
            .ok_or_else(|| panic!("No active guess"));

        let (guess_idx, guess) = active_guess?;
        let guess_idx = guess_idx as u16;

        if guess.remaining_letters() > 0 {
            return Err(GameError::IncompleteGuess);
        }

        let mut answer_chars: Vec<_> = answer.to_ascii_uppercase().chars().collect();
        let mut result = vec![LetterResult::Absent; game_options.word_length as usize];

        let guess_chars: Vec<_> = guess.as_chars();
        for i in 0..guess_chars.len() {
            if guess_chars[i] == answer_chars[i].to_ascii_uppercase() {
                result[i] = LetterResult::Correct;
                answer_chars[i] = char::MIN;
            }
        }

        for (i, &g) in guess_chars.iter().enumerate() {
            if g != char::MIN {
                if let Some(pos) = answer_chars.iter().position(|&a| a == g) {
                    result[i] = LetterResult::Present;
                    answer_chars[pos] = char::MIN;
                }
            }
        }

        guess.complete_guess(&result);

        if result.iter().all(|x| x == &LetterResult::Correct) {
            self.game_state = GameState::Won;
        } else if game_options.max_guesses - guess_idx - 1 > 0 {
            let mut next_guess = &mut self.guesses[(guess_idx + 1) as usize];
            next_guess.state = GuessState::Active;
        } else {
            self.game_state = GameState::Lost;
        }

        Ok(self.game_state)
    }

    fn clear(&mut self) {
        self.guesses =
            Guess::make_vec(self.game_options.word_length, self.game_options.max_guesses);
        self.game_state = GameState::Active;
    }
}
