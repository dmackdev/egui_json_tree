use std::collections::BTreeSet;

use serde_json::Value;

pub fn search(value: &Value, search_term: &String) -> BTreeSet<String> {
    let mut matching_paths = BTreeSet::new();

    if is_valid_search_term(search_term) {
        search_impl(value, search_term, &mut vec![], &mut matching_paths);
    }

    matching_paths
}

fn search_impl(
    value: &Value,
    search_term: &String,
    path_segments: &mut Vec<String>,
    matching_paths: &mut BTreeSet<String>,
) {
    match value {
        Value::Null => {
            if "null".contains(search_term) {
                update_matches(path_segments, matching_paths);
            }
        }
        Value::Bool(b) => {
            if b.to_string().contains(search_term) {
                update_matches(path_segments, matching_paths);
            }
        }
        Value::Number(n) => {
            if n.to_string().contains(search_term) {
                update_matches(path_segments, matching_paths);
            }
        }
        Value::String(s) => {
            if s.to_string().contains(search_term) {
                update_matches(path_segments, matching_paths);
            }
        }
        Value::Array(arr) => {
            for (idx, elem) in arr.iter().enumerate() {
                path_segments.push(idx.to_string());
                search_impl(elem, search_term, path_segments, matching_paths);
                path_segments.pop();
            }
        }
        Value::Object(obj) => {
            for (key, val) in obj.iter() {
                path_segments.push(key.to_string());
                if key.contains(search_term) {
                    update_matches(path_segments, matching_paths);
                }
                search_impl(val, search_term, path_segments, matching_paths);
                path_segments.pop();
            }
        }
    };
}

fn update_matches(path_segments: &mut Vec<String>, matching_paths: &mut BTreeSet<String>) {
    let mut path_str = "".to_string();
    matching_paths.insert(path_str);

    for i in 0..path_segments.len() {
        path_str = path_segments[0..i].join("/").to_string();
        matching_paths.insert(path_str);
    }
}

pub fn is_valid_search_term(search_term: &String) -> bool {
    !search_term.is_empty()
}
