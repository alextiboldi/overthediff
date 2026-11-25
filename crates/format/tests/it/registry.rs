//! Format resolution (story E02-S1, task T6).

use otd_format::{Case, Format, Kind, RegisterError, Registry};

fn registry(formats: &[(&str, Kind, &str)]) -> Registry {
    let mut registry = Registry::default();
    for &(name, kind, masks) in formats {
        registry.register_kind(kind);
        registry.register(Format::new(name, kind, masks).unwrap()).unwrap();
    }
    registry
}

fn resolved(registry: &Registry, file_name: &str) -> Option<String> {
    registry.resolve(file_name, Case::Sensitive).map(|f| f.name.clone())
}

#[test]
fn the_first_enabled_format_whose_mask_matches_wins_and_a_disabled_one_is_skipped() {
    let mut registry = registry(&[("A", Kind::Text, "*.txt"), ("B", Kind::Text, "*.*")]);
    assert_eq!(resolved(&registry, "x.txt").as_deref(), Some("A"));

    assert!(registry.set_enabled("A", false));
    assert_eq!(resolved(&registry, "x.txt").as_deref(), Some("B"));
}

#[test]
fn an_empty_mask_format_never_resolves_automatically_but_can_be_picked_by_name() {
    let registry = registry(&[("C", Kind::Text, ""), ("D", Kind::Text, " ; ;")]);
    for name in ["x.txt", "noext", ""] {
        assert_eq!(resolved(&registry, name), None, "{name:?}");
    }
    assert!(registry.by_name("C").is_some() && registry.by_name("D").is_some());
}

#[test]
fn a_features_formats_follow_the_built_ins_in_registration_order() {
    let mut registry = Registry::default();
    registry.register_kind(Kind::Text);
    let mut built_in = Format::new("Built-in", Kind::Text, "*.txt").unwrap();
    built_in.built_in = true;
    registry.register(built_in).unwrap();

    registry.register_kind(Kind::Table);
    for name in ["Feature 2", "Feature 1"] {
        registry.register(Format::new(name, Kind::Table, "*.csv").unwrap()).unwrap();
    }

    let names: Vec<_> = registry.formats().iter().map(|f| f.name.as_str()).collect();
    assert_eq!(names, ["Built-in", "Feature 2", "Feature 1"]);
    assert_eq!(resolved(&registry, "a.csv").as_deref(), Some("Feature 2"), "the earlier registration wins a tie");
}

#[test]
fn a_mask_compares_names_with_the_data_sources_case() {
    let registry = registry(&[("Text", Kind::Text, "*.txt")]);
    assert!(registry.resolve("README.TXT", Case::Insensitive).is_some());
    assert!(registry.resolve("README.TXT", Case::Sensitive).is_none());
}

#[test]
fn a_format_of_an_unregistered_kind_or_a_taken_name_is_refused() {
    let mut registry = registry(&[("Text", Kind::Text, "*.txt")]);
    let hex = Format::new("Hex", Kind::Hex, "*.bin").unwrap();
    assert_eq!(registry.register(hex), Err(RegisterError::KindNotRegistered(Kind::Hex)));

    let again = Format::new("Text", Kind::Text, "*.md").unwrap();
    assert_eq!(registry.register(again), Err(RegisterError::DuplicateName("Text".into())));
    assert_eq!(registry.formats().len(), 1, "a refused format is not added");
}
