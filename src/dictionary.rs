use once_cell::sync::Lazy;
use rand::prelude::IteratorRandom;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::fs;
use std::sync::Arc;
use std::sync::OnceLock;

/// Errors that can occur when working with dictionaries
#[derive(Debug)]
pub enum DictionaryError {
    /// Failed to load dictionary file
    FileLoadError,
    /// No word found matching the criteria
    WordNotFound,
}

impl Display for DictionaryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileLoadError => write!(f, "Failed to load dictionary file"),
            Self::WordNotFound => write!(f, "No word found matching the criteria"),
        }
    }
}

impl Error for DictionaryError {}

/// Dictionary containing words of a specific length
pub struct Dictionary {
    /// Name of the dictionary
    pub name: String,
    /// Length of words in this dictionary
    pub length: u8,
    /// Path to the dictionary file
    filename: String,
    /// Lazily loaded function to read words from file
    all_words: Lazy<Box<dyn Fn(&str) -> Result<Vec<String>, DictionaryError>>>,
}

impl Clone for Dictionary {
    fn clone(&self) -> Self {
        Dictionary {
            name: self.name.clone(),
            length: self.length,
            filename: self.filename.clone(),
            all_words: Lazy::new(|| Self::load_dictionary()),
        }
    }
}

impl Debug for Dictionary {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Dictionary")
            .field("name", &self.name)
            .field("length", &self.length)
            .field("filename", &self.filename)
            .finish()
    }
}

impl Display for Dictionary {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Dictionary: {}, Word Length: {}", self.name, self.length)
    }
}

impl Dictionary {
    /// Creates a function that loads words from a dictionary file
    fn load_dictionary() -> Box<dyn Fn(&str) -> Result<Vec<String>, DictionaryError>> {
        Box::new(|filename| {
            fs::read_to_string(filename)
                .map(|content| content.lines().map(String::from).collect())
                .map_err(|_| DictionaryError::FileLoadError)
        })
    }

    /// Creates a new dictionary
    fn new(name: &str, file: &str, count: u8) -> Self {
        Dictionary {
            name: name.to_string(),
            filename: file.to_string(),
            length: count,
            all_words: Lazy::new(|| Self::load_dictionary()),
        }
    }

    /// Gets a random word from the dictionary
    pub fn random_word(&self) -> Result<String, DictionaryError> {
        let func = &self.all_words;
        let contents = func(self.filename.as_str())?;

        contents
            .iter()
            .choose(&mut rand::rng())
            .cloned()
            .ok_or(DictionaryError::WordNotFound)
    }
}

thread_local! {
    // ideally this wouldn't be a thread local, but there doesn't seem to be any other way to make
    static DICTIONARY_CACHE: OnceLock<Vec<Arc<Dictionary>>> = OnceLock::new();
}

/// Gets all available dictionaries
///
/// Returns a vector of Arc pointers to dictionaries.
/// Since Arc is a reference-counted pointer, cloning it is cheap.
pub fn get_dictionaries() -> Vec<Arc<Dictionary>> {
    DICTIONARY_CACHE.with(|local| {
        local
            .get_or_init(|| {
                vec![
                    Arc::new(Dictionary::new("Wordle", "data/wordle.txt", 5)),
                    Arc::new(Dictionary::new("Scrabble", "data/scrabble.txt", 4)),
                    Arc::new(Dictionary::new("Scrabble", "data/scrabble.txt", 5)),
                    Arc::new(Dictionary::new("Scrabble", "data/scrabble.txt", 6)),
                    Arc::new(Dictionary::new("Scrabble", "data/scrabble.txt", 7)),
                    Arc::new(Dictionary::new("Dutch", "data/dutch.txt", 4)),
                    Arc::new(Dictionary::new("Dutch", "data/dutch.txt", 5)),
                    Arc::new(Dictionary::new("Dutch", "data/dutch.txt", 6)),
                    Arc::new(Dictionary::new("Dutch", "data/dutch.txt", 7)),
                    Arc::new(Dictionary::new("Dutch", "data/dutch.txt", 8)),
                    Arc::new(Dictionary::new("French", "data/french.txt", 4)),
                    Arc::new(Dictionary::new("French", "data/french.txt", 5)),
                    Arc::new(Dictionary::new("French", "data/french.txt", 6)),
                    Arc::new(Dictionary::new("French", "data/french.txt", 7)),
                    Arc::new(Dictionary::new("French", "data/french.txt", 8)),
                    Arc::new(Dictionary::new("Italian", "data/italian.txt", 4)),
                    Arc::new(Dictionary::new("Italian", "data/italian.txt", 5)),
                    Arc::new(Dictionary::new("Italian", "data/italian.txt", 6)),
                    Arc::new(Dictionary::new("Italian", "data/italian.txt", 7)),
                    Arc::new(Dictionary::new("Italian", "data/italian.txt", 8)),
                ]
            })
            .iter()
            .map(|dict| Arc::clone(dict))
            .collect()
    })
}
