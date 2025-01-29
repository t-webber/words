#![feature(let_chains)]

use crate::download::download_all;
use crate::parser::file_to_words;

mod download;
mod parser;

#[tokio::main]
async fn main() {
    println!("Extracting words...");
    let parsed_words = match file_to_words(&["01-10"]) {
        Ok(parsed_words) => parsed_words,
        Err(err) => panic!("Failed to parse html files.\n{err}"),
    };
    println!("Found {} words!", parsed_words.len());

    println!("Downloading definitions...");
    let _ = download_all(parsed_words)
        .await
        .unwrap_or_else(|err| panic!("{err:?}"));
}
