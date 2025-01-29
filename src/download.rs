use std::fs;

use crate::parser::ParsedWord;

#[derive(Debug)]
struct Html(String);

#[derive(Debug)]
pub struct DefinedWord {
    pub name: String,
    pub definition: Html,
}

const DEFS_PREFIX: &str = "./data/defs/";
const WIKI_PREFIX: &str = "https://en.wiktionary.org/";

pub async fn download_all(words: Vec<ParsedWord>) -> Result<Vec<DefinedWord>, String> {
    futures::future::join_all(words.into_iter().map(download_one))
        .await
        .into_iter()
        .collect()
}

async fn download_one(word: ParsedWord) -> Result<DefinedWord, String> {
    let path = format!("{DEFS_PREFIX}{}.html", word.name);
    let html = match fs::read_to_string(&path) {
        Ok(html) => Ok(html),
        Err(_) => {
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            let url = format!("{WIKI_PREFIX}{}", word.link);
            let response = reqwest::get(&url).await.map_err(|err| {
                format!(
                    "Error on word {}: failed to fetch response from server.\n{err}",
                    word.name
                )
            })?;
            println!("Downloaded word {} ({url} => {path})", &word.name);

            if response.status().is_success() {
                let html = response.text().await.map_err(|err| {
                    format!(
                        "Error on word {}: failed to convert response to text.\n{err}",
                        word.name
                    )
                })?;
                fs::write(path, &html).map_err(|err| {
                    format!(
                        "Error on word {}: failed to write response to fs.\n{err}",
                        word.name
                    )
                })?;
                println!("Downloading {word:?} was successful");
                Ok(html)
            } else {
                let err = format!("Request failed with status: {}", response.status());
                println!("[{:50}]{err}", word.name);
                Err(err)
            }
        }
    }?;

    Ok(DefinedWord {
        name: word.name,
        definition: Html(html),
    })
}
