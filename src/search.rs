use std::collections::BTreeSet;

use crate::value::{BaseValue, ExpandableType, JsonTreeValue};

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

    pub fn find_matching_paths_in(&self, value: &JsonTreeValue) -> BTreeSet<Vec<String>> {
        let mut matching_paths = BTreeSet::new();

        search_impl(value, self, &mut vec![], &mut matching_paths);

        if matching_paths.len() == 1 {
            // The only match was a top level key or value - no need to expand anything.
            matching_paths.clear();
        }

        matching_paths
    }

    fn matches<V: ToString + ?Sized>(&self, other: &V) -> bool {
        other.to_string().to_ascii_lowercase().contains(&self.0)
    }
}

fn search_impl(
    value: &JsonTreeValue,
    search_term: &SearchTerm,
    path_segments: &mut Vec<String>,
    matching_paths: &mut BTreeSet<Vec<String>>,
) {
    match value {
        JsonTreeValue::BaseValue(BaseValue { value_str, .. }) => {
            if search_term.matches(value_str) {
                update_matches(path_segments, matching_paths);
            }
        }
        JsonTreeValue::Expandable(entries, expandable_type) => {
            for (key, val) in entries.iter() {
                path_segments.push(key.to_string());

                // Ignore matches for indices in an array.
                if *expandable_type == ExpandableType::Object && search_term.matches(key) {
                    update_matches(path_segments, matching_paths);
                }

                search_impl(val, search_term, path_segments, matching_paths);
                path_segments.pop();
            }
        }
    };
}

fn update_matches(path_segments: &mut Vec<String>, matching_paths: &mut BTreeSet<Vec<String>>) {
    for i in 0..path_segments.len() {
        matching_paths.insert(path_segments[0..i].to_vec());
    }
}
