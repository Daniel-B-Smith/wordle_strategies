use std::fs::File;
use std::io::{BufRead, BufReader};

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

#[derive(Copy, Clone, Debug, PartialEq)]
enum Match {
    NoMatch,
    WrongPosition,
    Matched,
}

// This function is excessively complicated and slow due to the duplicate letter problem. I'm sure
// it can be substantially improved upon.
fn match_word(word: [u8; 5], guess: [u8; 5]) -> [Match; 5] {
    let mut out = [Match::NoMatch; 5];
    // First assign the matches without duplicates. We'll clean those up later.
    for i in 0..5 {
        if guess.iter().filter(|c| **c == guess[i]).count() > 1 {
            continue;
        }
        if word[i] == guess[i] {
            out[i] = Match::Matched;
        } else if word.iter().any(|c| *c == guess[i]) {
            out[i] = Match::WrongPosition;
        }
    }

    // If there are more matches in the guess than in the source word, we have to choose the
    // 'best' match. This is defined to be the first match(es) if both are in the wrong
    // position or the matching position regardless of order.
    //
    // TODO: We do this for every duplicated letter in the source.
    let mut match_counts = [0; 5];
    for (c, m) in word.iter().zip(&mut match_counts) {
        *m = guess.iter().filter(|g| **g == *c).count();
    }
    for i in 0..5 {
        if match_counts[i] < 2 {
            continue;
        }
        let w_count = word.iter().filter(|c| **c == word[i]).count();
        let mut set = 0;
        // Set the matches in order.
        for j in 0..5 {
            if set == w_count {
                break;
            }
            if guess[j] == word[i] && word[j] == guess[j] {
                out[j] = Match::Matched;
                set += 1;
            }
        }
        // Set the remaining wrong positions.
        for j in 0..5 {
            if set == w_count {
                break;
            }
            if guess[j] == word[i] && word[j] != guess[j] {
                out[j] = Match::WrongPosition;
                set += 1;
            }
        }
    }
    out
}

fn str_to_array(s: &str) -> [u8; 5] {
    let mut out: [u8; 5] = Default::default();
    for (c, o) in s.bytes().zip(&mut out) {
        *o = c;
    }
    out
}

fn filter_match_no_dup(word: [u8; 5], guess: [u8; 5], matches: [Match; 5]) -> bool {
    for i in 0..5 {
        if matches[i] == Match::Matched && word[i] != guess[i] {
            return false;
        }
        if matches[i] == Match::NoMatch && word.iter().any(|c| *c == guess[i]) {
            return false;
        }
        if matches[i] == Match::WrongPosition && word[i] == guess[i] {
            return false;
        }
        if matches[i] == Match::WrongPosition && !word.iter().any(|c| *c == guess[i]) {
            return false;
        }
    }

    true
}

// When there are duplicate characters, you have to consider the whole word an not just
// character by character.
fn filter_match_dup(word: [u8; 5], guess: [u8; 5], matches: [Match; 5]) -> bool {
    for i in 0..5 {
        if matches[i] == Match::Matched && word[i] != guess[i] {
            return false;
        }
        if matches[i] == Match::WrongPosition && word[i] == guess[i] {
            return false;
        }
        if matches[i] == Match::WrongPosition && !word.iter().any(|c| *c == guess[i]) {
            return false;
        }
        if matches[i] == Match::NoMatch && word[i] == guess[i] {
            return false;
        }
        if matches[i] == Match::NoMatch
            && guess.iter().filter(|c| **c == guess[i]).count() == 1
            && word.iter().any(|c| *c == guess[i])
        {
            return false;
        }
    }

    for i in 0..5 {
        for j in i + 1..5 {
            if guess[i] == guess[j] {
                // Check that there are enough matching letters in the guess to make the word a
                // possible match.
                let g_letters = guess.iter().filter(|c| **c == guess[i]).count();
                let w_letters = word.iter().filter(|c| **c == guess[i]).count();
                let matches = (0..5)
                    .filter(|k| guess[*k] == guess[i])
                    .filter(|k| {
                        matches[*k] == Match::Matched || matches[*k] == Match::WrongPosition
                    })
                    .count();
                if g_letters >= w_letters && matches < w_letters {
                    return false;
                }
            }
        }
    }
    true
}

fn has_duplicate(w: [u8; 5]) -> bool {
    for i in 0..5 {
        for j in i + 1..5 {
            if w[i] == w[j] {
                return true;
            }
        }
    }
    false
}

fn filter_match(word: [u8; 5], guess: [u8; 5], matches: [Match; 5]) -> bool {
    // TODO: This fast path might actually not be necessary...
    if has_duplicate(guess) {
        filter_match_dup(word, guess, matches)
    } else {
        filter_match_no_dup(word, guess, matches)
    }
}

fn filter_words(words: &mut Vec<[u8; 5]>, guess: [u8; 5], matches: [Match; 5]) {
    words.retain(|w| filter_match(*w, guess, matches));
}

/*
Maybe another bug:
Answer: ['u', 'n', 'c', 'l', 'e']
Seed: 4171687965805832080
*/

fn main() {
    let f = match File::open("wordle_words.txt") {
        Ok(f) => f,
        Err(why) => panic!("Failed to open wordle_words.txt: {}", why),
    };

    let words: Vec<_> = BufReader::new(f)
        .lines()
        .map(|l| str_to_array(&l.unwrap()))
        .collect();

    let seed = rand::random();
    println!("Seed: {}", seed);
    let mut rng = StdRng::seed_from_u64(seed);
    let mut counts = Vec::new();
    for _ in 0..1000_000 {
        let answer = words[rng.gen_range(0..words.len())];

        let mut guesses = words.clone();
        let mut rounds = 0;

        while guesses.len() > 1 {
            rounds += 1;
            let guess = guesses[rng.gen_range(0..guesses.len())];
            let matches = match_word(answer, guess);
            filter_words(&mut guesses, guess, matches);
        }

        if guesses.is_empty() {
            println!("answer: {:?}", answer);
        }
        assert_eq!(guesses.len(), 1);
        assert_eq!(guesses[0], answer);
        if counts.len() < rounds {
            counts.resize(rounds, 0);
        }
        counts[rounds - 1] += 1;
    }
    println!("Counts: {:?}", counts);
}

#[cfg(test)]
mod tests {
    use super::*;
    use Match::*;

    fn a(w: &str) -> [u8; 5] {
        str_to_array(w)
    }

    #[test]
    fn test_match_word() {
        assert_eq!(match_word(a("fates"), a("wrung")), [NoMatch; 5]);
        assert_eq!(
            match_word(a("fates"), a("facts")),
            [Matched, Matched, NoMatch, WrongPosition, Matched]
        );
        assert_eq!(
            match_word(a("phone"), a("photo")),
            [Matched, Matched, Matched, NoMatch, NoMatch]
        );
        assert_eq!(
            match_word(a("spawn"), a("floss")),
            [NoMatch, NoMatch, NoMatch, WrongPosition, NoMatch]
        );
        assert_eq!(
            match_word(a("brand"), a("await")),
            [NoMatch, NoMatch, Matched, NoMatch, NoMatch]
        );
        assert_eq!(
            match_word(a("bloom"), a("prowl")),
            [NoMatch, NoMatch, Matched, NoMatch, WrongPosition]
        );
    }

    fn cycle(m: Match) -> Match {
        match m {
            Match::Matched => Match::WrongPosition,
            Match::WrongPosition => Match::NoMatch,
            Match::NoMatch => Match::Matched,
        }
    }

    #[test]
    fn test_filter_match() {
        assert!(filter_match(
            a("fates"),
            a("plate"),
            [
                NoMatch,
                NoMatch,
                WrongPosition,
                WrongPosition,
                WrongPosition
            ]
        ));
        for i in 0..5 {
            let mut matches = [
                NoMatch,
                NoMatch,
                WrongPosition,
                WrongPosition,
                WrongPosition,
            ];
            matches[i] = cycle(matches[i]);
            assert!(
                !filter_match(a("fates"), a("plates"), matches),
                "{:?}",
                matches
            );
            matches[i] = cycle(matches[i]);
            assert!(
                !filter_match(a("fates"), a("plates"), matches),
                "{:?}",
                matches
            );
        }

        assert!(filter_match(
            a("photo"),
            a("spool"),
            [NoMatch, WrongPosition, Matched, WrongPosition, NoMatch,]
        ));

        for i in 0..5 {
            let mut matches = [NoMatch, WrongPosition, Matched, WrongPosition, NoMatch];
            matches[i] = cycle(matches[i]);
            assert!(
                !filter_match(a("photo"), a("spool"), matches),
                "{:?}",
                matches
            );

            matches[i] = cycle(matches[i]);
            assert!(
                !filter_match(a("photo"), a("spool"), matches),
                "{:?}",
                matches
            );
        }
        assert!(filter_match(
            a("brand"),
            a("await"),
            [NoMatch, NoMatch, Matched, NoMatch, NoMatch]
        ));
        assert!(filter_match(
            a("bunny"),
            a("nanny"),
            [NoMatch, NoMatch, Matched, Matched, Matched]
        ));
        assert!(filter_match(
            a("mamma"),
            a("gamma"),
            [NoMatch, Matched, Matched, Matched, Matched]
        ));
    }
}
