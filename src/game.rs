use std::cmp::PartialEq;

#[derive(Hash, Eq, PartialEq, Clone)]
pub enum LetterResult {
    Empty,
    Absent,
    Present,
    Correct,
}

#[derive(Default, Clone)]
pub struct GameOptions {
    dictionary_path: String,
    pub word_length: u16,
    pub max_guesses: u16,
}

impl GameOptions {
    pub fn default() -> Self {
        GameOptions {
            dictionary_path: String::from("scrabble.txt"),
            word_length: 5,
            max_guesses: 6,
        }
    }

    fn clone(&self) -> Self {
        GameOptions {
            dictionary_path: String::from(&self.dictionary_path),
            word_length: self.word_length,
            max_guesses: self.max_guesses,
        }
    }
    pub fn random_word(&self) -> &str {
        "elbow"
    }
}
#[derive(Clone, Eq, PartialEq)]
pub struct Guess {
    max_length: u16,
    letters: Vec<char>,
    result: Option<Vec<LetterResult>>,
}

impl Guess {
    fn new(max_length: u16) -> Self {
        Guess {
            max_length,
            letters: vec![],
            result: Option::None,
        }
    }

    fn add_letter(&mut self, c: char) {
        if (self.remaining_letters() > 0) {
            self.letters.push(c);
        }
    }

    fn delete_letter(&mut self) {
        if (!self.is_empty()) {
            self.letters.pop();
        }
    }

    pub fn is_empty(&self) -> bool {
        self.letters.len() == 0
    }

    pub fn is_complete(&self) -> bool {
        self.result.is_some()
    }

    fn remaining_letters(&self) -> u16 {
        self.max_length - (self.letters.len() as u16)
    }

    fn guess_string(&self) -> String {
        self.letters.iter().collect::<String>()
    }

    fn set_result(&mut self, result: &Vec<LetterResult>) {
        self.result = Option::Some(result.clone());
    }

    pub fn values(&self) -> Vec<(Option<char>, Option<LetterResult>)> {
        let mut result:Vec<(Option<char>, Option<LetterResult>)> = vec![];

        for i in 0..self.max_length {
            result[i as usize] = self.value_at(i);
        }

        result
    }

    pub fn value_at(&self, i: u16) -> (Option<char>, Option<LetterResult>) {
        let letters = &self.letters;

        if self.is_complete() {
            let result = self.result.as_ref().unwrap();
            (
                Option::Some(letters[i as usize]),
                Option::Some(result[i as usize].clone()),
            )
        } else {
            if (i < letters.len() as u16) {
                (Option::Some(letters[i as usize]), Option::None)
            } else {
                (Option::None, Option::None)
            }
        }
    }
}

#[derive(Eq, PartialEq)]
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
        let game_opts = opts.clone();

        let guesses = (0..opts.max_guesses)
            .map(|_| Guess::new(game_opts.word_length))
            .collect();

        GameData {
            game_state: GameState::Active,
            game_options: game_opts,
            answer: String::from(word),
            guesses,
            active_guess_idx: 0,
        }
    }

    fn active_guess(&mut self) -> Option<&mut Guess> {
        if self.game_state == GameState::Active {
            return Option::Some(&mut self.guesses[self.active_guess_idx as usize]);
        }

        return Option::None;
    }

    fn is_active(&self) -> bool {
        self.game_state == GameState::Active
    }

    pub fn add_letter(&mut self, val: char) -> bool {
        if (!self.is_active()) {
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
        if (!self.is_active()) {
            return false;
        }

        // can unrap here because we've checked the is_active
        let guess = self.active_guess().unwrap();
        if (guess.is_empty()) {
            return false;
        }

        guess.delete_letter();

        true
    }

    pub fn submit_word(&mut self) -> bool {
        if (!self.is_active()) {
            return false;
        }

        let letter_count = { self.game_options.word_length as usize };

        let answer = { self.answer.clone() };

        let guess = self.active_guess().unwrap();
        // can't complete a turn that is already complete
        if (guess.is_complete()) {
            return false;
        }

        // can't complete a turn that hasn't got the required number of letters
        if guess.remaining_letters() > 0 {
            return false;
        }

        let mut answer_chars: Vec<_> = { answer.chars().collect() };
        let mut result = vec![LetterResult::Absent; letter_count];

        let guess_chars: Vec<_> = { guess.guess_string().chars().collect() };


        // First pass: check for correct (green)
        for i in 0..guess_chars.len() {
            if guess_chars[i] == answer_chars[i] {
                result[i] = LetterResult::Correct;
                answer_chars[i] = char::MIN;
            }
        }

        // Second pass: check for present (yellow)
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

        // we've finished with that guess struct, move to the next one in the list for processing
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
