//! Name alignment within one folder (E03-S2-T3): the left and right listings of a folder pair up
//! by name, and an item with no partner is an orphan of its side.

use std::collections::HashMap;

use otd_source::{Capabilities, Item};

/// One aligned name: an item on each side, or on one side only (an orphan of that side).
#[derive(Debug, Clone, PartialEq)]
pub struct Aligned {
    /// The left item's name when there is one, otherwise the right's. Each side's item keeps its
    /// own spelling.
    pub name: String,
    pub left: Option<Item>,
    pub right: Option<Item>,
}

/// Whether names match case-sensitively when comparing these two sources. *Unspecified* (the
/// story only covers two sources with the same rule): when either side is case-insensitive,
/// matching is too, because that side keeps only one item for both spellings.
pub fn case_sensitive(left: &Capabilities, right: &Capabilities) -> bool {
    left.case_sensitive && right.case_sensitive
}

/// Pairs the items of one folder's left and right listings by name, in name order whatever order
/// the listings came in. An exact name match wins over a match that differs only in case, so on
/// a case-sensitive side holding `a.txt` and `A.txt`, `a.txt` pairs with `a.txt`. A file and a
/// folder with the same name pair up; telling them apart is the status model's job.
pub fn align(mut left: Vec<Item>, mut right: Vec<Item>, case_sensitive: bool) -> Vec<Aligned> {
    // Sorted first, so which of two case-variants pairs doesn't depend on listing order.
    left.sort_by(|a, b| a.name.cmp(&b.name));
    right.sort_by(|a, b| a.name.cmp(&b.name));
    let mut left: Vec<Option<Item>> = left.into_iter().map(Some).collect();
    let mut right: Vec<Option<Item>> = right.into_iter().map(Some).collect();
    let mut rows = Vec::new();
    pair_by(&mut left, &mut right, &mut rows, str::to_owned);
    if !case_sensitive {
        // ponytail: `to_lowercase` is Unicode lower-casing, as in `MemorySource`, not the OS's own
        // case table; E08-S4 owns the filename-case criteria.
        pair_by(&mut left, &mut right, &mut rows, str::to_lowercase);
    }
    rows.extend(left.into_iter().flatten().map(|item| Aligned { name: item.name.clone(), left: Some(item), right: None }));
    rows.extend(right.into_iter().flatten().map(|item| Aligned { name: item.name.clone(), left: None, right: Some(item) }));
    rows.sort_by(|a, b| a.name.cmp(&b.name));
    rows
}

/// Moves every left item whose key matches a still-unpaired right item's into `rows`, each right
/// item pairing at most once.
fn pair_by(left: &mut [Option<Item>], right: &mut [Option<Item>], rows: &mut Vec<Aligned>, key: impl Fn(&str) -> String) {
    let mut by_key: HashMap<String, Vec<usize>> = HashMap::new();
    // Pushed in reverse, so `pop` hands out the first of several right items with one key.
    for (index, item) in right.iter().enumerate().rev() {
        if let Some(item) = item {
            by_key.entry(key(&item.name)).or_default().push(index);
        }
    }
    for slot in left.iter_mut() {
        let Some(name_key) = slot.as_ref().map(|item| key(&item.name)) else { continue };
        let Some(partner) = by_key.get_mut(&name_key).and_then(Vec::pop).and_then(|index| right.get_mut(index)).and_then(Option::take) else {
            continue;
        };
        if let Some(item) = slot.take() {
            rows.push(Aligned { name: item.name.clone(), left: Some(item), right: Some(partner) });
        }
    }
}
