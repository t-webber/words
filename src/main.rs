#![feature(let_chains)]

use crate::download::download_all;
use crate::parser::file_to_words;

mod download;
mod parser;

const SECTIONS: [&str; 10] = [
    "01-10", "11-20", "21-30", "31-40", "41-50", "51-60", "61-70", "71-80", "81-90", "91-100",
];

#[tokio::main]
async fn main() {
    println!("Extracting words...");
    let parsed_words = match file_to_words(&SECTIONS) {
        Ok(parsed_words) => parsed_words,
        Err(err) => panic!("Failed to parse html files.\n{err}"),
    }
    .to_vec();

    println!("Found {} words!", parsed_words.len());

    println!("Downloading definitions...");
    let _ = download_all(parsed_words)
        .await
        .unwrap_or_else(|err| panic!("{err:?}"));
}
