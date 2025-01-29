use std::fs;

use crate::parser::ParsedWord;

pub fn make_lists(words: &[ParsedWord]) {
    make_list(&words, "all", |_| true).unwrap();
    make_list(&words, "all_valid", |word| !word.link.contains("index.php")).unwrap();
    make_list(&words, "alpha_lower", |word| {
        word.name.chars().all(|ch| ch.is_ascii_lowercase())
    })
    .unwrap();
    make_list(&words, "alpha_lower_valid", |word| {
        !word.link.contains("index.php") && word.name.chars().all(|ch| ch.is_ascii_lowercase())
    })
    .unwrap();
    make_list(&words, "min_3_alpha_lower", |word| {
        word.name.len() >= 3 && word.name.chars().all(|ch| ch.is_ascii_lowercase())
    })
    .unwrap();
    make_list(&words, "min_3_alpha_lower_valid", |word| {
        word.name.len() >= 3
            && !word.link.contains("index.php")
            && word.name.chars().all(|ch| ch.is_ascii_lowercase())
    })
    .unwrap();
}

fn make_list<F: Fn(&&ParsedWord) -> bool>(
    words: &[ParsedWord],
    path: &str,
    selector: F,
) -> Result<(), std::io::Error> {
    fs::write(
        format!("data/txt/{path}.txt"),
        words
            .iter()
            .filter(selector)
            .map(|word| word.name.to_owned())
            .collect::<Vec<String>>()
            .join("\n"),
    )
}
