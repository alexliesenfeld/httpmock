mod extract_example_tests;

use serde_json::json;
use std::fs;
use syn::{File, Item, ItemImpl};
use std::time::{SystemTime, UNIX_EPOCH};
use syn::spanned::Spanned;
use std::collections::BTreeMap;

fn main() {
    let file_content = fs::read_to_string("../src/api/spec.rs").expect("Unable to read file");
    let syntax_tree: File = syn::parse_file(&file_content).expect("Unable to parse file");

    let mut when_docs = BTreeMap::new();
    let mut then_docs = BTreeMap::new();

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

    fs::write("target/generated/docs.json", json_output_str).expect("Unable to write file");
}

fn extract_docs_for_impl(docs: &mut BTreeMap<String, String>, items: &Vec<syn::ImplItem>) {
    for item in items {
        if let syn::ImplItem::Method(method) = item {
            let method_name = method.sig.ident.to_string();
            let method_docs = extract_docs(&method.attrs);

            docs.insert(method_name, method_docs);
        }
    }
}

fn extract_docs(attrs: &Vec<syn::Attribute>) -> String {
    let mut doc_string = String::new();
    for attr in attrs {
        if attr.path.is_ident("doc") {
            if let Ok(meta) = attr.parse_meta() {
                if let syn::Meta::NameValue(nv) = meta {
                    if let syn::Lit::Str(lit) = nv.lit {
                        let trimmed_line = if lit.value().starts_with(' ') {
                            lit.value()[1..].to_owned()
                        } else {
                            lit.value().to_owned()
                        };
                        doc_string.push_str(&trimmed_line);
                        doc_string.push('\n');
                    }
                }
            }
        }
    }
    doc_string
}
