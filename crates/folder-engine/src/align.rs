//! Name alignment: within one folder (E03-S2-T3), the left and right listings of a folder pair up
//! by name, and an item with no partner is an orphan of its side. In flattened mode (E03-S2-T7),
//! the files of two whole trees pair up by name the same way, whatever subfolder each is in.

use std::collections::HashMap;
use std::hash::Hash;
use std::path::{Path, PathBuf};

use otd_source::{Capabilities, Item, ItemKind};

/// One aligned name: an item on each side, or on one side only (an orphan of that side). In
/// flattened mode each side is a [`Placed`] file, so its folder comes with it.
#[derive(Debug, Clone, PartialEq)]
pub struct Aligned<T = Item> {
    /// The left item's name when there is one, otherwise the right's. Each side's item keeps its
    /// own spelling.
    pub name: String,
    pub left: Option<T>,
    pub right: Option<T>,
}

/// An item of a tree with the folder it sits in, relative to the tree's base (empty for the base
/// itself), as a scan reports its folders.
#[derive(Debug, Clone, PartialEq)]
pub struct Placed {
    pub folder: PathBuf,
    pub item: Item,
}

trait Named {
    fn name(&self) -> &str;
    /// The folder a flattened file sits in; `None` within one folder.
    fn folder(&self) -> Option<&Path> {
        None
    }
}

impl Named for Item {
    fn name(&self) -> &str {
        &self.name
    }
}

impl Named for Placed {
    fn name(&self) -> &str {
        &self.item.name
    }
    fn folder(&self) -> Option<&Path> {
        Some(&self.folder)
    }
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
pub fn align(left: Vec<Item>, right: Vec<Item>, case_sensitive: bool) -> Vec<Aligned> {
    align_by_name(left, right, case_sensitive)
}

/// Flattened alignment: pairs the files of two whole trees by name, whatever folder each sits in,
/// with the same case rule and exact-match preference as [`align`]. Folders are left out, since
/// only files are aligned; links and unexpanded containers count as files.
///
/// *Unspecified*: when one side has several files with one name, a file at the same path on both
/// sides pairs first (when matching is case-insensitive, a path that differs only in case is the
/// same path). The others pair in folder order (`a/x.txt` before `b/x.txt`) with the other
/// side's remaining files of that name, and the rest are orphans of their side. The rows don't
/// depend on the order the items came in.
pub fn align_flat(mut left: Vec<Placed>, mut right: Vec<Placed>, case_sensitive: bool) -> Vec<Aligned<Placed>> {
    for side in [&mut left, &mut right] {
        side.retain(|placed| placed.item.kind != ItemKind::Folder);
        // The name sort in `align_by_name` is stable, so files of one name stay in folder order.
        side.sort_by(|a, b| a.folder.cmp(&b.folder));
    }
    align_by_name(left, right, case_sensitive)
}

fn align_by_name<T: Named>(mut left: Vec<T>, mut right: Vec<T>, case_sensitive: bool) -> Vec<Aligned<T>> {
    // Sorted first, so which of two case-variants pairs doesn't depend on listing order.
    left.sort_by(|a, b| a.name().cmp(b.name()));
    right.sort_by(|a, b| a.name().cmp(b.name()));
    let mut left: Vec<Option<T>> = left.into_iter().map(Some).collect();
    let mut right: Vec<Option<T>> = right.into_iter().map(Some).collect();
    let mut rows = Vec::new();
    // Same folder and name first: within one folder this is the exact-name pass, and when
    // flattened it keeps a file that didn't move paired with itself.
    pair_by(&mut left, &mut right, &mut rows, |item| (item.folder().map(Path::to_path_buf), item.name().to_owned()));
    // ponytail: `to_lowercase` is Unicode lower-casing, as in `MemorySource`, not the OS's own case
    // table; E08-S4 owns the filename-case criteria.
    if !case_sensitive {
        // On a case-insensitive side a path that differs only in case is the same path, so it
        // pairs before any cross-folder match. Within one folder this is the case-insensitive pass.
        pair_by(&mut left, &mut right, &mut rows, |item| (item.folder().map(|f| f.to_string_lossy().to_lowercase()), item.name().to_lowercase()));
    }
    pair_by(&mut left, &mut right, &mut rows, |item| item.name().to_owned());
    if !case_sensitive {
        pair_by(&mut left, &mut right, &mut rows, |item| item.name().to_lowercase());
    }
    rows.extend(left.into_iter().flatten().map(|item| Aligned { name: item.name().to_owned(), left: Some(item), right: None }));
    rows.extend(right.into_iter().flatten().map(|item| Aligned { name: item.name().to_owned(), left: None, right: Some(item) }));
    rows.sort_by(|a, b| a.name.cmp(&b.name));
    rows
}

/// Moves every left item whose key matches a still-unpaired right item's into `rows`, each right
/// item pairing at most once.
fn pair_by<T: Named, K: Eq + Hash>(left: &mut [Option<T>], right: &mut [Option<T>], rows: &mut Vec<Aligned<T>>, key: impl Fn(&T) -> K) {
    let mut by_key: HashMap<K, Vec<usize>> = HashMap::new();
    // Pushed in reverse, so `pop` hands out the first of several right items with one key.
    for (index, item) in right.iter().enumerate().rev() {
        if let Some(item) = item {
            by_key.entry(key(item)).or_default().push(index);
        }
    }
    for slot in left.iter_mut() {
        let Some(name_key) = slot.as_ref().map(&key) else { continue };
        let Some(partner) = by_key.get_mut(&name_key).and_then(Vec::pop).and_then(|index| right.get_mut(index)).and_then(Option::take) else {
            continue;
        };
        if let Some(item) = slot.take() {
            rows.push(Aligned { name: item.name().to_owned(), left: Some(item), right: Some(partner) });
        }
    }
}
