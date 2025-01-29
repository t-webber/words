use std::{
    fs,
    rc::{self, Rc},
};

use html5ever::{QualName, serialize::SerializeOpts, tendril::TendrilSink as _};
use markup5ever_rcdom::{Node, NodeData, RcDom, SerializableHandle};

use crate::{download::DefinedWord, parser::ParsedWord};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

fn remove_heading(tree: &RcDom) -> Vec<Rc<Node>> {
    let document = tree.document.as_ref().children.take();
    let html = {
        let first = document.first().unwrap().as_ref().children.take();
        if first.is_empty() {
            document.get(1).unwrap().as_ref().children.take()
        } else {
            first
        }
    };
    let mut html_iter = html.into_iter().skip(1);
    let mut next = html_iter.next().unwrap();
    while let NodeData::Text { contents } = &next.data
        && contents
            .borrow()
            .to_string()
            .chars()
            .all(char::is_whitespace)
    {
        next = html_iter.next().unwrap();
    }
    next.as_ref().children.take()
}

fn print_node(node: &Rc<Node>) -> String {
    let mut output = Vec::new();
    let handle = SerializableHandle::from(node.clone());
    html5ever::serialize(&mut output, &handle, SerializeOpts::default());
    String::from_utf8(output).unwrap()
}

fn print_nodes(nodes: &[Rc<Node>]) -> String {
    let mut output = String::new();
    for node in nodes {
        output.push_str(&print_node(node));
    }
    output
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

const DEFINITIONS_PREFIX: &str = "data/extracted/";

pub fn extract_all(words: Vec<DefinedWord>) -> Result<(), String> {
    for word in words {
        let output = format!("{DEFINITIONS_PREFIX}{}.html", word.name);
        if fs::read_to_string(&output).is_err() {
            let html = fs::read_to_string(&word.path).unwrap();
            let extracted = extract_one(&html)?;
            fs::write(output, extracted)
                .map_err(|err| format!("Failed to write to file.\n{err}"))?;
        }
    }
    Ok(())
}

fn extract_one(html: &str) -> Result<String, String> {
    let tree = html5ever::parse_document(
        markup5ever_rcdom::RcDom::default(),
        html5ever::ParseOpts::default(),
    )
    .from_utf8()
    .read_from(&mut html.as_bytes())
    .map_err(|err| format!("Invalid HTML string: {err}"))?;

    let body = remove_heading(&tree);

    // panic!("raw:\n{}", print_nodes(&body));

    let definitions = get_definitions(body, "English");

    Ok(print_nodes(&definitions))
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

fn get_definitions(nodes: Vec<rc::Rc<Node>>, section: &str) -> Vec<rc::Rc<Node>> {
    if search_title_with_depth(2, &nodes, section) {
        nodes
    } else {
        nodes
            .into_iter()
            .flat_map(|child| get_definitions(child.children.take(), section))
            .collect()
    }
}

fn is_section(node: &Node, section: &str) -> bool {
    if let NodeData::Element { name, attrs, .. } = &node.data
        && name.local.to_string() == "h2"
        && attrs
            .borrow()
            .iter()
            .any(|x| x.name.local.to_string() == "id" && x.value.to_string() == section)
    {
        true
    } else {
        false
    }
}

fn search_title_with_depth(depth: usize, nodes: &[rc::Rc<Node>], section: &str) -> bool {
    if depth == 0 && nodes.iter().any(|node| is_section(node, section)) {
        true
    } else if depth == 0 {
        false
    } else {
        for node in nodes {
            if search_title_with_depth(depth - 1, &node.children.borrow(), section) {
                // println!("{} [{depth}]\n{node:#?}", ">".repeat(depth * 4));
                return true;
            }
        }
        false
    }
}
