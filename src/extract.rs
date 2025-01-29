use std::{fs, rc};

use html5ever::tendril::TendrilSink as _;
use markup5ever_rcdom::{Node, NodeData};

pub fn extract_all() {
    let html = fs::read_to_string("test.html").unwrap();
    extract_one(&html, "out.html").unwrap();
}

fn extract_one(html: &str, outfilename: &str) -> Result<String, String> {
    let tree = html5ever::parse_document(
        markup5ever_rcdom::RcDom::default(),
        html5ever::ParseOpts::default(),
    )
    .from_utf8()
    .read_from(&mut html.as_bytes())
    .map_err(|err| format!("Invalid HTML string: {err}"))?;

    let definitions = get_definitions(tree.document);
    for node in definitions {
        println!(
            "================================================\n{node:#?}\n================================================"
        );
    }

    Ok(String::new())
}

fn get_definitions(node: rc::Rc<Node>) -> Vec<rc::Rc<Node>> {
    if search_title_with_depth(2, &node) {
        vec![node]
    } else {
        node.children
            .take()
            .into_iter()
            .flat_map(get_definitions)
            .collect()
    }
}

fn search_title_with_depth(depth: usize, node: &rc::Rc<Node>) -> bool {
    println!("{}[{depth}]\t{node:#?}", "-".repeat(depth * 4));
    if let NodeData::Element { name, attrs, .. } = &node.data
        && name.local.to_string() == "h2"
        && attrs
            .borrow()
            .iter()
            .any(|x| x.name.local.to_string() == "id" && x.value.to_string() == "English")
    {
        // println!("true");
        // true
        panic!();
    } else if depth == 0 {
        false
    } else {
        node.children
            .borrow()
            .iter()
            .any(|child| search_title_with_depth(depth - 1, node))
    }
}
