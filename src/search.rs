use std::collections::HashSet;

use egui::Id;

use crate::{
    pointer::JsonPointerSegment,
    value::{ExpandableType, JsonTreeValue, ToJsonTreeValue},
};

#[derive(Debug, Clone, Hash)]
pub struct SearchTerm(String);

impl SearchTerm {
    pub(crate) fn parse(search_str: &str) -> Option<Self> {
        SearchTerm::is_valid(search_str).then_some(Self(search_str.to_ascii_lowercase()))
    }

    fn is_valid(search_str: &str) -> bool {
        !search_str.is_empty()
    }

    pub(crate) fn find_match_indices_in(&self, other: &str) -> Vec<usize> {
        other
            .to_ascii_lowercase()
            .match_indices(&self.0)
            .map(|(idx, _)| idx)
            .collect()
    }

    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    pub(crate) fn find_matching_paths_in<'a, T: ToJsonTreeValue>(
        &self,
        value: &'a T,
        abbreviate_root: bool,
        make_persistent_id: &dyn Fn(&[JsonPointerSegment]) -> Id,
    ) -> HashSet<Id> {
        let mut matching_paths = HashSet::new();

        search_impl(
            value,
            self,
            &mut vec![],
            &mut matching_paths,
            make_persistent_id,
        );

        if !abbreviate_root && matching_paths.len() == 1 {
            // The only match was a top level key or value - no need to expand anything.
            matching_paths.clear();
        }

        matching_paths
    }

    fn matches<V: ToString + ?Sized>(&self, other: &V) -> bool {
        other.to_string().to_ascii_lowercase().contains(&self.0)
    }
}

fn search_impl<'a, T: ToJsonTreeValue>(
    value: &'a T,
    search_term: &SearchTerm,
    path_segments: &mut Vec<JsonPointerSegment<'a>>,
    matching_paths: &mut HashSet<Id>,
    make_persistent_id: &dyn Fn(&[JsonPointerSegment]) -> Id,
) {
    match value.to_json_tree_value() {
        JsonTreeValue::Base(_, display_value, _) => {
            if search_term.matches(display_value) {
                update_matches(path_segments, matching_paths, make_persistent_id);
            }
        }
        JsonTreeValue::Expandable(entries, expandable_type) => {
            for (property, val) in entries.iter() {
                path_segments.push(*property);

                // Ignore matches for indices in an array.
                if expandable_type == ExpandableType::Object && search_term.matches(property) {
                    update_matches(path_segments, matching_paths, make_persistent_id);
                }

                search_impl(
                    *val,
                    search_term,
                    path_segments,
                    matching_paths,
                    make_persistent_id,
                );
                path_segments.pop();
            }
        }
    };
}

fn update_matches<'a>(
    path_segments: &[JsonPointerSegment<'a>],
    matching_paths: &mut HashSet<Id>,
    make_persistent_id: &dyn Fn(&[JsonPointerSegment]) -> Id,
) {
    for i in 0..path_segments.len() {
        matching_paths.insert(make_persistent_id(&path_segments[0..i]));
    }
}
