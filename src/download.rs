use std::fs;

use crate::parser::ParsedWord;

type Html = String;

#[derive(Debug)]
pub struct DefinedWord {
    pub name: String,
    pub definition: Html,
}

const DEFS_PREFIX: &str = "./data/defs/";
const WIKI_PREFIX: &str = "https://en.wiktionary.org/";

pub async fn download_all(words: Vec<ParsedWord>) -> Result<Vec<DefinedWord>, String> {
    let mut res = Vec::with_capacity(words.len());
    for word in words {
        if !word.link.contains("index.php") {
            res.push(download_one(word).await?);
        }
    }
    Ok(res)
}

async fn download_one(word: ParsedWord) -> Result<DefinedWord, String> {
    let path = format!("{DEFS_PREFIX}{}.html", word.name);
    let html = match fs::read_to_string(&path) {
        Ok(html) => {
            println!("Word {} already downloaded", &word.name);
            html
        }
        Err(_) => {
            let url = format!("{WIKI_PREFIX}{}", word.link);
            let html = fetch_bounce_back(&url, &word.name).await?;
            println!("Downloaded word {} ({url} => {path})", &word.name);
            fs::write(path, &html).map_err(|err| {
                format!(
                    "Error on word {}: failed to write response to fs.\n{err}",
                    word.name
                )
            })?;
            html
        }
    };

    Ok(DefinedWord {
        name: word.name,
        definition: html,
    })
}

async fn fetch_bounce_back(url: &str, name: &str) -> Result<Html, String> {
    let mut current_sleep = 100u64; // 0.1 seconds
    let max_sleep = 600_000; // 10 minutes

    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(current_sleep)).await;
        let response = reqwest::get(url).await.map_err(|err| {
            format!("Error on word {name}: failed to fetch response from server.\n{err}",)
        })?;

        if response.status().is_success() {
            return response.text().await.map_err(|err| {
                format!("Error on word {name}: failed to convert response to text.\n{err}")
            });
        } else if response.status().as_u16() != 429 {
            let err = format!("Request failed with status: {}", response.status());
            println!("[{name:50}]{err}");
            return Err(err);
        }

        if current_sleep > max_sleep {
            return Err("Error code 429: Too many requests.".to_owned());
        }

        println!("Too many requests, retrying");
        current_sleep *= 2;
    }
}
