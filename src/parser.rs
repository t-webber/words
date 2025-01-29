use html5ever::ParseOpts;
use html5ever::tendril::TendrilSink as _;
use markup5ever_rcdom::Node;
use markup5ever_rcdom::NodeData;
use std::fs;

#[derive(Debug)]
pub struct ParsedWord {
    pub name: String,
    pub link: String,
}

fn make_error(node: &Node, msg: &str) -> String {
    format!("\nInvalid node.\n>>> {msg}\nNode: {node:?}")
}

impl ParsedWord {
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
        Ok(Some(ParsedWord { link, name }))
    }
}

fn html_to_words(html: &str) -> Result<Vec<ParsedWord>, String> {
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
    let mut words: Vec<ParsedWord> = Vec::with_capacity(tags.len());
    for (i, tag) in tags.iter().enumerate() {
        if let Some(word) =
            ParsedWord::from_tag(tag).map_err(|err| format!("--- [{i}] ---\n{err}"))?
        {
            #[cfg(feature = "debug")]
            println!("ParsedWord = {:?}", word);
            words.push(word);
        }
    }
    Ok(words)
}

pub fn file_to_words(paths: &[&str]) -> Result<Vec<ParsedWord>, String> {
    let mut words = Vec::with_capacity(paths.len() * 10_000);
    for path in paths {
        let full_path = format!("./data/lists/{path}.html");
        let html = fs::read_to_string(&full_path)
            .map_err(|err| format!("Invalid path {full_path}: {err}"))?;
        words.extend(html_to_words(&html).map_err(|err| format!("--- [{path}] ---\n{err}"))?);
    }
    Ok(words)
}
