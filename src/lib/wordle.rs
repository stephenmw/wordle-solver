use std::fmt;

use anyhow::{bail, Result};

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct Word([u8; 5]);

impl std::str::FromStr for Word {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        if s.len() != 5 {
            bail!("unexpected word length {}: `{}`", s.len(), s);
        }
        let mut word = [0; 5];
        word.as_mut_slice().copy_from_slice(s.as_bytes());
        Ok(Word(word))
    }
}

impl fmt::Display for Word {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = std::str::from_utf8(self.0.as_slice()).expect("Word must contain valid UTF-8");
        write!(f, "{}", s)
    }
}

impl fmt::Debug for Word {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum State {
    CorrectLocation,
    IncorrectLocaiton,
    NotExists,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CompareResult(u8);

impl From<[State; 5]> for CompareResult {
    fn from(s: [State; 5]) -> Self {
        let n = s.iter().copied().fold(0, |acc, x| acc * 3 + x as u8);
        Self(n)
    }
}

impl Into<usize> for CompareResult {
    fn into(self) -> usize {
        self.0 as usize
    }
}

pub fn compare_words(guess: &Word, actual: &Word) -> CompareResult {
    let mut actual = actual.0;
    let mut result = [State::NotExists; 5];

    for i in 0..result.len() {
        if guess.0[i] == actual[i] {
            actual[i] = 0;
            result[i] = State::CorrectLocation;
        }
    }

    for i in 0..result.len() {
        if result[i] == State::NotExists {
            if let Some(j) = actual.iter().position(|&x| x == guess.0[i]) {
                actual[j] = 0;
                result[i] = State::IncorrectLocaiton;
            };
        }
    }

    result.into()
}
