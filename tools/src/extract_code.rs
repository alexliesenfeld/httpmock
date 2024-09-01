use serde_json::json;
use std::fs;
use syn::{File, Item, ItemImpl};
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    let file_content = fs::read_to_string("../src/api/spec.rs").expect("Unable to read file");
    let syntax_tree: File = syn::parse_file(&file_content).expect("Unable to parse file");

    let mut when_docs = serde_json::Map::new();
    let mut then_docs = serde_json::Map::new();

    for item in syntax_tree.items {
        if let Item::Impl(ItemImpl { self_ty, items, .. }) = &item {
            if let syn::Type::Path(type_path) = &**self_ty {
                let ident = &type_path.path.segments.last().unwrap().ident;
                if ident == "When" {
                    extract_docs_for_impl(&mut when_docs, items);
                } else if ident == "Then" {
                    extract_docs_for_impl(&mut then_docs, items);
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

    fs::write("target/generated/code_examples.json", json_output_str).expect("Unable to write file");
}

fn extract_docs_for_impl(docs: &mut serde_json::Map<String, serde_json::Value>, items: &Vec<syn::ImplItem>) {
    for item in items {
        if let syn::ImplItem::Method(method) = item {
            if let Some(example) = extract_code_example(&method.attrs) {
                docs.insert(method.sig.ident.to_string(), json!(example));
            }
        }
    }
}

fn extract_code_example(attrs: &Vec<syn::Attribute>) -> Option<String> {
    let mut example = String::new();
    let mut in_code_block = false;

    for attr in attrs {
        if attr.path.is_ident("doc") {
            if let Ok(meta) = attr.parse_meta() {
                if let syn::Meta::NameValue(nv) = meta {
                    if let syn::Lit::Str(lit) = nv.lit {
                        let doc_line = lit.value();
                        if doc_line.trim().starts_with("```rust") {
                            example.push_str("```rust\n");
                            in_code_block = true;
                        } else if doc_line.trim().starts_with("```") && in_code_block {
                            example.push_str("```\n");
                            in_code_block = false;
                        } else if in_code_block {
                            example.push_str(&doc_line);
                            example.push('\n');
                        }
                    }
                }
            }
        }
    }

    if example.is_empty() {
        None
    } else {
        Some(example)
    }
}
