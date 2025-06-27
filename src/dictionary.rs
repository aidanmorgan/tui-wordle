use once_cell::sync::Lazy;
use rand::prelude::IteratorRandom;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::fs;
use std::rc::Rc;
use std::sync::{OnceLock};

#[derive(Debug)]
pub enum DictionaryError {
    FileLoadError,
    NoMatchingName,
    NoMatchingLength,
    CacheError
}
impl Display for DictionaryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for DictionaryError {}

pub struct Dictionary {
    pub name: String,
    pub length: u8,

    filename: String,

    // this is a gross trick to try and defer the loading of the file into memory as late as possible
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
        write!(f, "Dictionary[{},{},{}]", self.name, self.length, self.filename)
    }
}

impl Display for Dictionary {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Dictionary: {}, Word Length: {}", self.name, self.length)
    }
}

impl Dictionary {
    fn load_dictionary() -> Box<dyn Fn(&str) -> Result<Vec<String>, DictionaryError>> {
        return Box::new(|x| {
            let val = fs::read_to_string(x);

            match val {
                Ok(_) => {
                    return Ok(val.unwrap().lines().map(|x| String::from(x)).collect());
                }
                Err(_) => {
                    return Err(DictionaryError::FileLoadError);
                }
            }
        });
    }
    fn new(name: &str, file: &str, count: u8) -> Self {
        Dictionary {
            name: String::from(name),
            filename: String::from(file),
            length: count,
            all_words: Lazy::new(|| Self::load_dictionary()),
        }
    }
    pub fn random_word(&self) -> Result<String, Box<dyn Error>> {
        let func = &self.all_words;

        let contents = func(self.filename.as_str())?;
        
        let filtered = contents.iter()
            .filter(|x| x.len() == self.length as usize)
            .choose(&mut rand::rng())
            .unwrap();

        return Ok(filtered.clone());
    }
}



thread_local! {
    // ideally this wouldn't be a thread local, but there doesn't seem to be any other way to make
    static DICTIONARY_CACHE: OnceLock<Vec<Rc<Dictionary>>> = OnceLock::new();
}


// Intentionally not returning a reference this cache, it's cloning Rc's so it doesnt matter if
// we lose ownership of it and let the caller decide when to clean up
pub fn get_dictionaries() -> Vec<Rc<Dictionary>> {
    return DICTIONARY_CACHE.with(|local| {
        local.get_or_init(|| {
            vec![
                Rc::new(Dictionary::new("Wordle", "data/wordle.txt", 5)),
                Rc::new(Dictionary::new("Scrabble", "data/scrabble.txt", 4)),
                Rc::new(Dictionary::new("Scrabble", "data/scrabble.txt", 5)),
                Rc::new(Dictionary::new("Scrabble", "data/scrabble.txt", 6)),
                Rc::new(Dictionary::new("Scrabble", "data/scrabble.txt", 7)),
                Rc::new(Dictionary::new("Dutch", "data/dutch.txt", 5)),
                Rc::new(Dictionary::new("French", "data/french.txt", 6)),
                Rc::new(Dictionary::new("French", "data/french.txt", 7)),
                Rc::new(Dictionary::new("French", "data/french.txt", 8)),
                Rc::new(Dictionary::new("Italian", "data/italian.txt", 5))
            ]
        }).clone()
    });
}
