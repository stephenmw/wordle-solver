use crate::lib::wordle::{compare_words, Word};

#[derive(Clone)]
pub struct Game<'a> {
    pub guesses: Vec<Word>,
    pub words_left: Vec<Word>,
    pub answer: Word,
    state: GameState,
    all_words: &'a [Word],
}

impl<'a> Game<'a> {
    pub fn new(words: &'a [Word], answer: &Word) -> Self {
        Game {
            guesses: Vec::with_capacity(6),
            words_left: words.to_vec(),
            all_words: words,
            answer: *answer,
            state: GameState::Running,
        }
    }

    pub fn reset_with_answer(&mut self, new_answer: &Word) {
        self.guesses.truncate(0);
        if self.words_left.len() != self.all_words.len() {
            self.words_left.truncate(0);
            self.words_left.extend_from_slice(self.all_words);
        }
        self.state = GameState::Running;
        self.answer = *new_answer
    }

    pub fn guess(&mut self, w: &Word) -> GameState {
        if self.state != GameState::Running {
            return self.state;
        }

        self.guesses.push(*w);

        let res = compare_words(w, &self.answer);
        self.words_left.retain(|x| compare_words(w, x) == res);

        let new_state = {
            if w == &self.answer {
                GameState::Success
            } else if self.guesses.len() >= 6 {
                GameState::Failure
            } else {
                GameState::Running
            }
        };

        self.state = new_state;
        new_state
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameState {
    Running,
    Success,
    Failure,
}
