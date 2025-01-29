#![allow(unused)]
#![feature(let_chains)]

use crate::download::download_all;
use crate::parser::file_to_words;

mod download;
mod extract;
mod parser;

const SECTIONS: [&str; 16] = [
    "001-010", "011-020", "021-030", "031-040", "041-050", "051-060", "061-070", "071-080",
    "081-090", "091-100", "101-110", "111-120", "121-130", "131-140", "141-150", "151-157",
];

#[tokio::main]
async fn main() {
    match main_wrapper().await {
        Ok(()) => (),
        Err(err) => panic!("An error occurred.\n\n{err:?}"),
    }
}

async fn main_wrapper() -> Result<(), String> {
    println!("Extracting words...");
    let parsed_words = file_to_words(&SECTIONS)?.to_vec();
    println!("Extracted {} words!", parsed_words.len());

    println!("Fetching definitions...");
    let defined_words = download_all(parsed_words).await?;
    println!("Fetched {} words!", defined_words.len());

    println!("Extracting definitions...");
    extract::extract_all(defined_words);
    Ok(())
}
