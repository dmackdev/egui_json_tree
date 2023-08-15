use std::collections::BTreeSet;

use serde_json::Value;

#[derive(Debug, Clone)]
pub struct SearchTerm(String);

impl SearchTerm {
    pub fn parse(search_term: String) -> Option<Self> {
        SearchTerm::is_valid(&search_term).then_some(Self(search_term.to_ascii_lowercase()))
    }

    fn is_valid(search_str: &String) -> bool {
        !search_str.is_empty()
    }

    pub fn match_index(&self, other: &str) -> Option<usize> {
        other
            .to_ascii_lowercase()
            .match_indices(&self.0)
            .next()
            .map(|(idx, _)| idx)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn find_matching_paths_in(&self, value: &Value) -> BTreeSet<String> {
        let mut matching_paths = BTreeSet::new();

        search_impl(value, self, &mut vec![], &mut matching_paths);

        matching_paths
    }

    fn matches<V: ToString + ?Sized>(&self, other: &V) -> bool {
        other.to_string().to_ascii_lowercase().contains(&self.0)
    }
}

const NULL_STR: &str = "null";

fn search_impl(
    value: &Value,
    search_term: &SearchTerm,
    path_segments: &mut Vec<String>,
    matching_paths: &mut BTreeSet<String>,
) {
    match value {
        Value::Null => {
            if search_term.matches(NULL_STR) {
                update_matches(path_segments, matching_paths);
            }
        }
        Value::Bool(b) => {
            if search_term.matches(b) {
                update_matches(path_segments, matching_paths);
            }
        }
        Value::Number(n) => {
            if search_term.matches(n) {
                update_matches(path_segments, matching_paths);
            }
        }
        Value::String(s) => {
            if search_term.matches(s) {
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
                if search_term.matches(key) {
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
