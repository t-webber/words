use std::{
    cell::{Cell, RefCell},
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
    let len = words.len();

    let mut err = vec![];
    for (i, word) in words.into_iter().enumerate() {
        let output = format!("{DEFINITIONS_PREFIX}{}.html", word.name);
        if extract_word(&word.path, &output).is_err() {
            println!("[{i}]\t {}\tfailed", word.name);
            err.push(word);
        }
    }
    panic!(
        "Errors = {}\n\nNumber of errors: {}/{}",
        err.iter()
            .map(|word| word.name.to_owned())
            .collect::<Vec<String>>()
            .join(" "),
        err.len(),
        len
    );
    Ok(())

    // extract_word("test.html", "out.html")
}

fn extract_word(input: &str, output: &str) -> Result<(), String> {
    if fs::read_to_string(output).is_err() {
        let html = fs::read_to_string(input).unwrap();
        let extracted = extract_one(&html)?;
        fs::write(output, extracted).map_err(|err| format!("Failed to write to file.\n{err}"))?;
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

    let definitions = {
        let en = get_definitions(&body, "English");
        if !en.is_empty() {
            en
        } else {
            dbg!(&body);
            /// TODO: data taken even with clone so this doesn't work
            get_definitions(&body, "Translingual")
        }
    };

    if definitions.is_empty() {
        return Err("Failed to find English or Translingual section".to_string());
    }

    Ok(print_nodes(&(definitions)))
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

fn get_definitions(nodes: &Vec<Rc<Node>>, section: &str) -> Vec<Rc<Node>> {
    let search = search_title_with_depth(2, nodes, section);
    match search {
        SearchStatus::Node(selected_nodes) => selected_nodes,
        SearchStatus::None => {
            for node in nodes {
                let rec = get_definitions(&node.to_owned().children.take(), section);
                if !rec.is_empty() {
                    return rec;
                }
            }
            vec![]
        }
        _ => panic!("invalid depths"),
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

enum SearchStatus {
    None,
    FoundTitle,
    FirstDepth(Vec<Rc<Node>>),
    Node(Vec<Rc<Node>>),
}

// const SEP: &str =
//     "-------------------------------------------------------------------------------------";

fn search_title_with_depth(depth: usize, nodes: &[Rc<Node>], section: &str) -> SearchStatus {
    if depth == 0 {
        for node in nodes.iter() {
            if is_section(node, section) {
                return SearchStatus::FoundTitle;
            }
        }
        SearchStatus::None
    } else {
        let mut iter = nodes.iter().enumerate();
        while let Some((idx, node)) = iter.next() {
            // println!(
            //     "{SEP}\n[{depth}]\t{:?}\n{SEP}",
            //     if let NodeData::Element { name, .. } = &node.data {
            //         format!("{}", name.local)
            //     } else {
            //         format!("{:?}", node.data)
            //     }
            // );
            let search =
                search_title_with_depth(depth - 1, node.children.borrow().as_ref(), section);
            return match search {
                SearchStatus::None => {
                    // println!("continue");
                    continue;
                }
                SearchStatus::FoundTitle => {
                    let mut title_and_next = Vec::with_capacity(1 + iter.len());

                    title_and_next.push(node.to_owned());
                    title_and_next.extend(
                        iter.map(|(_, next)| next.to_owned())
                            .collect::<Vec<Rc<Node>>>(),
                    );

                    // println!("found title, saving node\n{title_and_next:#?}");
                    SearchStatus::FirstDepth(title_and_next)
                }
                SearchStatus::FirstDepth(selected) => {
                    // println!("child is title, returning value\n{selected:#?}");

                    // let mut selected_and_next = Vec::with_capacity(1 + iter.len());

                    // selected_and_next.push(selected);

                    // // dbg!(&iter);

                    // selected_and_next.extend(
                    //     iter.map(|(_, next)| next.to_owned())
                    //         .collect::<Vec<Rc<Node>>>(),
                    // );

                    // println!(
                    //     ">>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>> {} at index {idx}",
                    //     selected_and_next.len()
                    // );

                    // println!(
                    //     "{SEP}\nDepth 2 found title:\nNode:\n{:?}\n{SEP}\nChildren:\n{:#?}\n{SEP}",
                    //     node.data, selected_and_next
                    // );

                    SearchStatus::Node(selected)
                }
                SearchStatus::Node(_) => panic!("Too deep"),
            };
        }
        SearchStatus::None
    }
}
