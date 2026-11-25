//! Name alignment within one folder, on `MemorySource` fixtures whose case rule comes from their
//! capabilities.

use std::path::Path;

use otd_folder_engine::{Aligned, align, case_sensitive};
use otd_source::{DataSource, Item, Links, MemorySource};

use crate::fixture::{at, caps};

fn source(case_sensitive: bool, names: &[&str]) -> MemorySource {
    names.iter().fold(MemorySource::new(caps(case_sensitive)), |source, name| source.file(name, *name, at(1)))
}

fn root(source: &MemorySource) -> Vec<Item> {
    source.list(Path::new(""), Links::Report).expect("the root lists")
}

/// Aligns the two roots the way the engine would: with the case rule the sources report.
fn align_roots(left: &MemorySource, right: &MemorySource) -> Vec<Aligned> {
    align(root(left), root(right), case_sensitive(&left.capabilities(), &right.capabilities()))
}

/// Each row's (left name, right name).
fn pairs(rows: &[Aligned]) -> Vec<(Option<&str>, Option<&str>)> {
    rows.iter().map(|row| (row.left.as_ref().map(|i| i.name.as_str()), row.right.as_ref().map(|i| i.name.as_str()))).collect()
}

#[test]
fn names_differing_only_in_case_align_on_case_insensitive_sources_and_not_on_case_sensitive_ones() {
    let insensitive = align_roots(&source(false, &["Readme.txt"]), &source(false, &["README.txt"]));
    assert_eq!(pairs(&insensitive), [(Some("Readme.txt"), Some("README.txt"))]);
    let sensitive = align_roots(&source(true, &["Readme.txt"]), &source(true, &["README.txt"]));
    assert_eq!(pairs(&sensitive), [(None, Some("README.txt")), (Some("Readme.txt"), None)]);
}

#[test]
fn every_listed_item_lands_in_exactly_one_row_and_an_unmatched_one_is_an_orphan_of_its_side() {
    let rows = align_roots(&source(true, &["a", "both"]), &source(true, &["both", "c"]));
    assert_eq!(pairs(&rows), [(Some("a"), None), (Some("both"), Some("both")), (None, Some("c"))]);
}

/// The unspecified mixed case: a case-insensitive side matches case-insensitively, and an exact
/// spelling on the case-sensitive side wins over a case-variant.
#[test]
fn an_exact_name_wins_over_a_case_variant_when_either_source_is_case_insensitive() {
    let left = source(false, &["readme.txt"]);
    let right = source(true, &["README.txt", "readme.txt"]);
    assert_eq!(pairs(&align_roots(&left, &right)), [(None, Some("README.txt")), (Some("readme.txt"), Some("readme.txt"))]);
    assert_eq!(pairs(&align_roots(&right, &left)), [(Some("README.txt"), None), (Some("readme.txt"), Some("readme.txt"))]);
}

/// `Ab` has two case-variants and no exact match, so only sorting keeps its partner fixed.
#[test]
fn listing_order_does_not_change_the_rows() {
    let left = root(&source(false, &["Ab", "c"]));
    let right = root(&source(true, &["aB", "C", "AB"]));
    let reversed = |items: &[Item]| items.iter().rev().cloned().collect::<Vec<_>>();
    assert_eq!(align(left.clone(), right.clone(), false), align(reversed(&left), reversed(&right), false));
}
