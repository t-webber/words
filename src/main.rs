#![feature(let_chains)]
#![allow(unused)]

use html5ever::ParseOpts;
use html5ever::QualName;
use html5ever::parse_document;
use html5ever::tendril::TendrilSink as _;
use markup5ever_rcdom::Node;
use markup5ever_rcdom::NodeData;
use markup5ever_rcdom::RcDom;
use std::fs;

#[derive(Debug)]
struct Word {
    name: String,
    link: String,
}

fn make_error(node: &Node, msg: &str) -> String {
    format!("\nInvalid node.\n>>> {msg}\nNode: {node:?}")
}

impl Word {
    fn from_tag(tag: &Node) -> Result<Option<Self>, String> {
        let link = if let NodeData::Element { attrs, .. } = &tag.data {
            attrs
                .borrow()
                .iter()
                .find(|attr| &attr.name.local.to_owned() == "href")
                .ok_or_else(|| make_error(tag, "Node doesn't contain a href attribute"))
                .map(|attr| attr.value.to_string())
        } else if let NodeData::Text { contents } = &tag.data
            && contents
                .take()
                .to_string()
                .chars()
                .all(|ch| ch.is_whitespace())
        {
            return Ok(None);
        } else {
            Err(make_error(tag, "Node data isn't an element"))
        }?;
        let name = if let NodeData::Text { contents } = &tag
            .children
            .take()
            .first()
            .ok_or_else(|| make_error(tag, "Node doesn't contain any children"))?
            .data
        {
            Ok(contents.take().to_ascii_lowercase())
        } else {
            Err(make_error(tag, "Node first child isn't text"))
        }?;
        Ok(Some(Word { link, name }))
    }
}

fn html_to_words(html: &str) -> Result<Vec<Word>, (String)> {
    let tree = html5ever::parse_document(markup5ever_rcdom::RcDom::default(), ParseOpts::default())
        .from_utf8()
        .read_from(&mut html.as_bytes())
        .map_err(|err| format!("Invalid HTML string: {err}"))?;
    let tags = tree
        .document
        .children
        .take()
        .first()
        .unwrap()
        .children
        .take()
        .get(1)
        .unwrap()
        .children
        .take();
    let mut words: Vec<Word> = Vec::with_capacity(tags.len());
    for (i, tag) in tags.iter().enumerate() {
        if let Some(word) = Word::from_tag(tag).map_err(|err| format!("--- [{i}] ---\n{err}"))? {
            println!("Word = {:?}", word);
            words.push(word);
        }
    }
    Ok(words)
}

fn file_to_words(paths: &[&str]) -> Result<Vec<Word>, String> {
    let mut words = Vec::with_capacity(paths.len() * 10_000);
    for path in paths {
        let html = fs::read_to_string(format!("data/wiki/{path}.html")).unwrap();
        words.extend(html_to_words(&html).map_err(|err| format!("--- [{path}] ---\n{err}"))?);
    }
    Ok(words)
}

fn main() {
    match file_to_words(&[
        "01-10", "11-20", "21-30", "31-40", "41-50", "51-60", "61-70", "71-80", "81-90", "91-100",
    ]) {
        Ok(words) => {
            println!("\n>>> len = {}", words.len());
        }
        Err(err) => eprintln!("An error occurred.\n{err}"),
    }
}
