use std::cmp::PartialEq;
use std::error;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;
use crate::dictionary::{get_dictionaries, Dictionary};

#[derive(Debug)]
pub enum GameError {
    FileLoadError,
    DictionaryError
}
impl error::Error for GameError {}

impl Display for GameError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}


#[derive(Hash, Eq, PartialEq, Clone, Debug)]
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
    pub dictionary: Rc<Dictionary>
}


impl Clone for GameOptions {
    fn clone(&self) -> Self {
        GameOptions {
            dictionary: Rc::clone(&self.dictionary),
            max_guesses: self.max_guesses,
            word_length: self.word_length
        }
    }
}

impl Default for GameOptions {
    fn default() -> Self {
        let dictionaries = get_dictionaries();

        let default_dictionary = dictionaries.iter()
            .filter(|x| x.name == "Wordle")
            .next();

        if let Some(x) = default_dictionary {
            Self {
                word_length: 5,
                max_guesses: 6,
                dictionary: Rc::clone(x)
            }
        }
        else {
            panic!("No default dictionary found")
        }
    }
}

impl GameOptions {
    pub fn random_word(&self) -> Result<String, GameError> {
        let randon_word = self.dictionary.random_word();

        if let Ok(y) = randon_word {
            return Ok(y);
        }
        else {
            return Err(GameError::DictionaryError);
        }

    }

    fn set_dictionary(&mut self, name: &str, length: u8) -> Result<(), GameError>{
        let dictionaries = get_dictionaries();

        if let Some(x) =  dictionaries.iter()
            .filter(|x| x.name == name && x.length == length)
            .next() {

            self.dictionary = Rc::clone(x);

            Ok(())
        }
        else {
            Err(GameError::DictionaryError)
        }
    }

}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Guess {
    max_length: u16,
    letters: Vec<char>,
    result: Option<Vec<LetterResult>>,
}

impl Guess {
    fn new(max_length: u16) -> Self {
        Self {
            max_length,
            letters: Vec::new(),
            result: None,
        }
    }

    fn add_letter(&mut self, c: char) {
        if self.remaining_letters() > 0 {
            self.letters.push(c);
        }
    }

    fn delete_letter(&mut self) {
        if !self.is_empty() {
            self.letters.pop();
        }
    }

    pub fn is_empty(&self) -> bool {
        self.letters.is_empty()
    }

    pub fn is_complete(&self) -> bool {
        self.result.is_some()
    }

    fn remaining_letters(&self) -> u16 {
        self.max_length - (self.letters.len() as u16)
    }

    fn guess_string(&self) -> String {
        self.letters.iter().collect()
    }

    fn set_result(&mut self, result: &Vec<LetterResult>) {
        self.result = Some(result.clone());
    }

    pub fn values(&self) -> Vec<(Option<char>, Option<LetterResult>)> {
        (0..self.max_length)
            .map(|i| self.value_at(i))
            .collect()
    }

    pub fn value_at(&self, i: u16) -> (Option<char>, Option<LetterResult>) {
        if self.is_complete() {
            let result = self.result.as_ref().unwrap();
            (
                self.letters.get(i as usize).copied(),
                result.get(i as usize).cloned(),
            )
        } else {
            if (i < self.letters.len() as u16) {
                (Some(self.letters[i as usize]), None)
            } else {
                (None, None)
            }
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
enum GameState {
    Active,
    Won,
    Lost,
}

pub struct GameData {
    game_state: GameState,
    game_options: GameOptions,
    answer: String,

    active_guess_idx: u16,
    pub guesses: Vec<Guess>,
}

impl GameData {
    pub fn new(opts: &GameOptions) -> Self {
        let word = opts.random_word();
        if(word.is_err()) {
            panic!()
        }

        let game_opts = opts.clone();

        let guesses = (0..opts.max_guesses)
            .map(|_| Guess::new(game_opts.word_length))
            .collect();

        Self {
            game_state: GameState::Active,
            game_options: game_opts,
            answer: word.unwrap().clone(),
            guesses,
            active_guess_idx: 0,
        }
    }

    fn active_guess(&mut self) -> Option<&mut Guess> {
        if matches!(self.game_state, GameState::Active) {
            Some(&mut self.guesses[self.active_guess_idx as usize])
        } else {
            None
        }
    }

    fn is_active(&self) -> bool {
        matches!(self.game_state, GameState::Active)
    }

    pub fn add_letter(&mut self, val: char) -> bool {
        if !self.is_active() {
            return false;
        }

        let guess = self.active_guess().unwrap();
        if guess.is_complete() || guess.remaining_letters() == 0 {
            return false;
        }

        guess.add_letter(val.to_ascii_uppercase());
        false
    }

    pub fn delete_letter(&mut self) -> bool {
        if !self.is_active() {
            return false;
        }

        let guess = self.active_guess().unwrap();
        if (guess.is_empty()) {
            return false;
        }

        guess.delete_letter();

        true
    }

    pub fn submit_word(&mut self) -> bool {
        if !self.is_active() {
            return false;
        }

        let letter_count = { self.game_options.word_length as usize };

        let answer = { self.answer.clone() };

        let guess = self.active_guess().unwrap();
        if (guess.is_complete()) {
            return false;
        }

        if guess.remaining_letters() > 0 {
            return false;
        }

        let mut answer_chars: Vec<_> = { answer.to_ascii_uppercase().chars().collect() };
        let mut result = vec![LetterResult::Absent; letter_count];

        let guess_chars: Vec<_> = { guess.guess_string().chars().collect() };
        for i in 0..guess_chars.len() {
            if guess_chars[i] == answer_chars[i] {
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

        guess.set_result(&result);

        if result.iter().all(|x| *x == LetterResult::Correct) {
            self.game_state = GameState::Won;
        } else {
        }

        self.active_guess_idx += 1;
        if self.active_guess_idx == self.game_options.max_guesses {
            self.game_state = GameState::Won;
        } else if (self.active_guess_idx > self.game_options.max_guesses) {
            self.game_state = GameState::Lost;
        }

        true
    }

    fn clear(&mut self) -> bool {
        self.guesses = (0..self.game_options.max_guesses)
            .map(|_| Guess::new(self.game_options.word_length))
            .collect();
        self.active_guess_idx = 0;
        self.game_state = GameState::Active;

        true
    }
}
