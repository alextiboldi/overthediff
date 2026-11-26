//! Name alignment within one folder, on `MemorySource` fixtures whose case rule comes from their
//! capabilities.

use std::path::Path;

use otd_folder_engine::{Aligned, Alignment, align, case_sensitive};
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
    align(root(left), root(right), Alignment::new(case_sensitive(&left.capabilities(), &right.capabilities())))
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
    assert_eq!(align(left.clone(), right.clone(), Alignment::new(false)), align(reversed(&left), reversed(&right), Alignment::new(false)));
}

/// A name with precomposed `é` (NFC) and its twin spelt `e` plus a combining acute (NFD), as
/// macOS and Linux file systems may each hold it. Escapes, so no editor or file system changes them.
const NFC: &str = "caf\u{e9}.txt";
const NFD: &str = "cafe\u{301}.txt";

fn with(extensions: bool, normalization: bool) -> Alignment {
    Alignment { extensions, normalization, ..Alignment::new(true) }
}

fn align_names(left: &[&str], right: &[&str], alignment: Alignment) -> Vec<Aligned> {
    align(root(&source(true, left)), root(&source(true, right)), alignment)
}

#[test]
fn names_differing_only_in_extension_share_a_row_only_with_the_extension_switch_on() {
    assert_eq!(pairs(&align_names(&["a.txt"], &["a.csv"], with(true, false))), [(Some("a.txt"), Some("a.csv"))]);
    assert_eq!(pairs(&align_names(&["a.txt"], &["a.csv"], with(false, true))), [(None, Some("a.csv")), (Some("a.txt"), None)]);
}

#[test]
fn an_nfc_name_and_its_nfd_twin_share_a_row_only_with_the_normalization_switch_on() {
    assert_eq!(pairs(&align_names(&[NFC], &[NFD], with(false, true))), [(Some(NFC), Some(NFD))]);
    assert_eq!(pairs(&align_names(&[NFC], &[NFD], with(true, false))).len(), 2, "two orphans");
}

/// An exact name pairs before a twin in another form, and both before a name with another
/// extension, so switching the options on never breaks up a pair that matched without them.
#[test]
fn a_closer_match_wins_over_a_looser_one() {
    let rows = align_names(&["a.csv", NFC], &[NFD, "a.txt", "caf\u{e9}.csv", NFC], with(true, true));
    assert_eq!(pairs(&rows), [(Some("a.csv"), Some("a.txt")), (None, Some(NFD)), (None, Some("caf\u{e9}.csv")), (Some(NFC), Some(NFC))]);
}

/// Two folders with dated names, or `README` and `README.md`, are different things, not one item
/// with another extension.
#[test]
fn the_extension_switch_pairs_only_files_that_both_have_an_extension() {
    let left = MemorySource::new(caps(true)).folder("release.2024", at(1)).file("README", "x", at(1)).file(".bashrc", "x", at(1));
    let right = MemorySource::new(caps(true)).folder("release.2025", at(1)).file("README.md", "x", at(1)).file(".profile", "x", at(1));
    let rows = align(root(&left), root(&right), with(true, false));
    assert!(rows.iter().all(|row| row.left.is_none() || row.right.is_none()), "{:?}", pairs(&rows));
}
