use std::fs;

use crate::parser::ParsedWord;

#[derive(Debug)]
pub struct DefinedWord {
    pub name: String,
    pub path: String,
}

fn word_to_url(word: &ParsedWord) -> String {
    format!("{WIKI_PREFIX}{}", word.link)
}
fn word_to_path(word: &ParsedWord) -> String {
    format!("{DEFS_PREFIX}{}.html", word.name)
}

const DEFS_PREFIX: &str = "./data/defs/";
const WIKI_PREFIX: &str = "https://en.wiktionary.org/";

pub async fn download_all(words: Vec<ParsedWord>) -> Result<Vec<DefinedWord>, String> {
    #[cfg(feature = "download")]
    {
        // let mut res = Vec::with_capacity(words.len());
        for word in words {
            if !word.link.contains("index.php") {
                // res.push(
                let _ = download_one(word).await?;
                // );
            } else {
                println!("Ignoring word {}", &word.name);
            }
        }
        Err("Download finished successfully".to_string())
    }
    #[cfg(not(feature = "download"))]
    {
        let mut res = Vec::with_capacity(words.len());
        for word in words {
            let path = word_to_path(&word);
            if fs::read_to_string(&path).is_ok() {
                res.push(DefinedWord {
                    name: word.name,
                    path,
                })
            }
        }
        Ok(res)
    }
}

async fn download_one(word: ParsedWord) -> Result<(), String> {
    let path = word_to_path(&word);
    match fs::read_to_string(&path) {
        Ok(html) => Ok(()),
        Err(_) => {
            let url = word_to_url(&word);
            let html = fetch_bounce_back(&url, &word.name).await?;
            println!("Downloaded word {} ({url} => {path})", &word.name);
            fs::write(path, html).map_err(|err| {
                format!(
                    "Error on word {}: failed to write response to fs.\n{err}",
                    word.name
                )
            })?;
            Ok(())
        }
    }
}

async fn fetch_bounce_back(url: &str, name: &str) -> Result<String, String> {
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
