mod lib;

use std::cmp::Ordering;
use std::collections::HashMap;
use std::env;
use std::fmt::Write;
use std::fs;

use anyhow::Result;
use rayon::prelude::*;

use lib::game::{Game, GameState};
use lib::parser;
use lib::wordle::{compare_words, CompareResult, Word};

fn main() {
    let args: Vec<_> = env::args().collect();

    let board_filename = match args.get(1) {
        Some(x) => x,
        None => panic!("Must give arg"),
    };

    match board_filename.as_str() {
        "test_all" => test_all(),
        "test_zonae" => test_word("zonae"),
        _ => next_board(board_filename),
    }
}

fn test_word(s: &str) {
    let words = load_words().expect("failed to load words");
    let answer: Word = s.parse().unwrap();
    let mut g = Game::new(&words, &answer);

    run_game_with_answer(&mut g, &answer);
    println!("{:?}", g.guesses)
}

fn next_board(filename: &str) {
    let words = load_words().expect("failed to load words");

    let board_raw = fs::read_to_string(filename).expect("failed to load board");
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
}

fn test_all() {
    let words = load_words().expect("failed to load words");

    let results: Vec<_> = words
        .par_iter()
        .map_with(Game::new(&words, &words[0]), |mut g, w| {
            run_game_with_answer(&mut g, w);
            (w, g.guesses.clone())
        })
        .collect();

    let mut output = String::new();

    for (w, guesses) in results.iter() {
        output
            .write_fmt(format_args!("{}:{}", w, guesses[0]))
            .unwrap();
        for g in &guesses[1..] {
            output.write_fmt(format_args!(",{}", g)).unwrap();
        }
        output.write_char('\n').unwrap();
    }

    print!("{}", output);
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
        .min_by(|a, b| compare_floats(a.1, b.1).then_with(|| a.0.cmp(&b.0)))
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

    ret.sort_by(|a, b| compare_floats(a.1, b.1).then_with(|| a.0.cmp(&b.0)));
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

fn compare_floats(a: f64, b: f64) -> Ordering {
    if (a - b).abs() < 0.0000000001 {
        Ordering::Equal
    } else {
        a.partial_cmp(&b).unwrap()
    }
}
