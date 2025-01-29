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
    #[cfg(feature = "download")]
    {
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
    #[cfg(not(feature = "download"))]
    extract::extract_all();
}
