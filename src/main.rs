mod lib;

use std::collections::HashMap;
use std::env;
use std::fs;

use anyhow::Result;
use rayon::prelude::*;

use lib::game::{Game, GameState};
use lib::parser;
use lib::wordle::{compare_words, State, Word};

type CompareResult = [State; 5];

fn main() {
    let args: Vec<_> = env::args().collect();

    let board_filename = match args.get(1) {
        Some(x) => x,
        None => panic!("Must specify board file"),
    };

    let words = load_words().expect("failed to load words");

    if board_filename != "test_all" {
        let board_raw = fs::read_to_string(board_filename).expect("failed to load board");
        let board = parser::parse_board(&board_raw)
            .expect("failed to parse board")
            .1;

        let mut possible_words = words.clone();
        for (guess, res) in board {
            possible_words.retain(|x| compare_words(&guess, x) == res);
        }

        for (w, cost) in best_starting_words(&possible_words) {
            println!("{}: {}", w, cost);
        }

        return;
    }

    let _results: Vec<_> = words
        .par_iter()
        .map_with(Game::new(&words, &words[0]), |mut g, w| {
            let res = run_game_with_answer(&mut g, w);
            if res == false {
                println!("{}: {} {:?}", w, res, g.guesses);
            }
            (w, res)
        })
        .collect();
}

fn load_words() -> Result<Vec<Word>> {
    let file = fs::read_to_string("words.txt")?;
    let mut words = Vec::new();
    for w in file.lines() {
        let word: Word = w.parse()?;
        words.push(word);
    }

    words.sort();

    Ok(words)
}

fn run_game_with_answer(g: &mut Game, answer: &Word) -> bool {
    g.reset_with_answer(answer);
    g.guess(&"rales".parse().unwrap());
    loop {
        let next = *best_next_word(&g.words_left);
        match g.guess(&next) {
            GameState::Success => return true,
            GameState::Failure => return false,
            GameState::Running => (), // no-op
        };
    }
}

fn best_next_word(words: &[Word]) -> &Word {
    let mut result_freq = HashMap::new();
    let avg = words.len() as f64 / (3 as f64).powf(5.0);

    words
        .iter()
        .map(|w| {
            find_results_distribution(w, words, &mut result_freq);
            let cost: f64 = result_freq
                .iter()
                .map(|(_, &c)| (avg - c as f64).powf(2.0))
                .sum();
            (w, cost)
        })
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .unwrap()
        .0
}

fn best_starting_words(words: &[Word]) -> Vec<(Word, f64)> {
    let all_results: Vec<_> = words
        .par_iter()
        .map_with(HashMap::new(), |mut freq, w| {
            find_results_distribution(w, &words, &mut freq);
            let res: Vec<_> = freq
                .drain()
                //.filter(|(x, _)| x != &[State::CorrectLocation; 5])
                .collect();
            (w, res)
        })
        .collect();

    let mut ret: Vec<_> = all_results
        .iter()
        .map(|(w, res)| {
            let avg = res.iter().map(|(_, c)| c).sum::<usize>() as f64 / res.len() as f64;
            let cost: f64 = res.iter().map(|&(_, c)| (avg - c as f64).powf(2.0)).sum();
            (**w, cost)
        })
        .collect();

    ret.sort_by(|(_, a), (_, b)| a.partial_cmp(&b).unwrap());
    ret
}

fn find_results_distribution(
    next_word: &Word,
    words: &[Word],
    out: &mut HashMap<CompareResult, usize>,
) {
    out.clear();

    for other in words {
        let s = compare_words(next_word, other);
        *out.entry(s).or_insert(0) += 1;
    }
}
