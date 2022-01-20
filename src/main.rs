mod lib;

use std::collections::HashMap;
use std::fs;

use anyhow::Result;
use rayon::prelude::*;

use lib::wordle::{compare_words, State, Word};

type CompareResult = [State; 5];

fn main() {
    let words = load_words().expect("failed to load words");

    for (w, cost) in best_starting_words(&words) {
        println!("{}: {}", w, cost);
    }
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

fn best_starting_words(words: &[Word]) -> Vec<(Word, f64)> {
    let all_results: Vec<_> = words
        .par_iter()
        .map(|w| {
            let res: Vec<_> = find_results_with_starting(w, &words)
                .into_iter()
                .filter(|(x, _)| x != &[State::CorrectLocation; 5])
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

fn find_results_with_starting(
    starting_word: &Word,
    words: &[Word],
) -> HashMap<CompareResult, usize> {
    let mut freq_table = HashMap::new();

    for other in words {
        let s = compare_words(starting_word, other);
        *freq_table.entry(s).or_insert(0) += 1;
    }

    freq_table
}
