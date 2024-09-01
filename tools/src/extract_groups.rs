use serde_json::json;
use std::fs;
use syn::{File, Item, ItemImpl};
use std::time::{SystemTime, UNIX_EPOCH};
use syn::spanned::Spanned;

fn main() {
    let file_content = fs::read_to_string("../src/api/spec.rs").expect("Unable to read file");
    let syntax_tree: File = syn::parse_file(&file_content).expect("Unable to parse file");

    let mut when_docs = vec![];
    let mut then_docs = vec![];

    for item in syntax_tree.items {
        if let Item::Impl(ItemImpl { self_ty, items, .. }) = &item {
            if let syn::Type::Path(type_path) = &**self_ty {
                let ident = &type_path.path.segments.last().unwrap().ident;
                if ident == "When" {
                    extract_docs_for_impl(&mut when_docs, items, &file_content);
                } else if ident == "Then" {
                    extract_docs_for_impl(&mut then_docs, items, &file_content);
                }
            }
        }
    }

    let json_output = json!({
        "when": when_docs,
        "then": then_docs
    });

    let json_output_str = serde_json::to_string_pretty(&json_output).expect("Unable to serialize JSON");
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    fs::write("target/generated/groups.json", json_output_str).expect("Unable to write file");
}

fn extract_docs_for_impl(docs: &mut Vec<serde_json::Value>, items: &Vec<syn::ImplItem>, file_content: &str) {
    for item in items {
        if let syn::ImplItem::Method(method) = item {
            let method_name = method.sig.ident.to_string();
            let method_span = method.span().end();
            let line_number = method_span.line - 1;

            println!("Processing method: {}", method_name);

            if let Some(group) = extract_group_marker(file_content, line_number) {
                docs.push(json!({
                    "method": method_name,
                    "group": group,
                }));
            } else {
                println!("No group marker found for method: {}", method_name);
                docs.push(json!({
                    "method": method_name,
                    "group": "No group",
                }));
            }
        }
    }
}

fn extract_group_marker(file_content: &str, line_number: usize) -> Option<String> {
    let lines: Vec<&str> = file_content.lines().collect();
    if line_number + 1 < lines.len() {
        let marker_line = lines[line_number + 1].trim();
        if marker_line.starts_with("// @docs-group:") {
            return Some(marker_line.trim_start_matches("// @docs-group:").trim().to_string());
        }
    }
    None
}