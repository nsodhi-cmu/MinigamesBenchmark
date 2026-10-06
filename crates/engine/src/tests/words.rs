use super::{WORDS, signature};

#[test]
fn every_word_is_five_lower_case_letters() {
    for word in WORDS {
        assert_eq!(word.len(), 5, "{word} is not five letters");
        assert!(word.bytes().all(|b| b.is_ascii_lowercase()), "{word} is not plain lower case");
    }
}

#[test]
fn no_word_appears_twice() {
    let mut seen = WORDS.to_vec();
    let total = seen.len();
    seen.sort_unstable();
    seen.dedup();
    assert_eq!(seen.len(), total, "a word is listed more than once");
}

/// What the reading games lean on: loose letters name exactly one word here,
/// so nothing has to check a dealt choice against the answer.
#[test]
fn no_two_words_are_spelled_with_the_same_letters() {
    let mut sets: Vec<[u8; 5]> = WORDS.iter().map(|word| signature(word)).collect();
    let total = sets.len();
    sets.sort_unstable();
    sets.dedup();
    assert_eq!(sets.len(), total, "two words share their letters, so the letters name two answers");
}

/// What the hiding game leans on: a word laid on a grid reads in one direction,
/// so where it was found is where it was put.
#[test]
fn no_word_reads_the_same_backwards() {
    for word in WORDS {
        let backwards: String = word.chars().rev().collect();
        assert_ne!(&backwards, word, "{word} reads the same either way");
    }
}
