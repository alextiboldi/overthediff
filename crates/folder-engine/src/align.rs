//! Name alignment: within one folder (E03-S2-T3), the left and right listings of a folder pair up
//! by name, and an item with no partner is an orphan of its side. In flattened mode (E03-S2-T7),
//! the files of two whole trees pair up by name the same way, whatever subfolder each is in.
//! The Comparison tab's alignment switches (E08-S4) let names that differ in extension or in
//! Unicode normalization form pair up too.

use std::borrow::Cow;
use std::collections::HashMap;
use std::hash::Hash;
use std::path::{Path, PathBuf};

use icu_normalizer::ComposingNormalizerBorrowed;
use otd_source::{Capabilities, Item, ItemKind};

/// How names match when items align.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Alignment {
    /// Names that differ only in case are different names: see [`case_sensitive`].
    pub case_sensitive: bool,
    /// *Align filenames with different extensions*: `a.txt` pairs with `a.csv`.
    pub extensions: bool,
    /// *Align filenames with different Unicode normalization forms*: a name spelled with
    /// precomposed characters (NFC) pairs with its decomposed twin (NFD).
    pub normalization: bool,
}

impl Alignment {
    /// Names match by the case rule alone, with both switches off.
    pub fn new(case_sensitive: bool) -> Self {
        Self { case_sensitive, extensions: false, normalization: false }
    }
}

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
    fn is_folder(&self) -> bool;
    /// The folder a flattened file sits in; `None` within one folder.
    fn folder(&self) -> Option<&Path> {
        None
    }
}

impl Named for Item {
    fn name(&self) -> &str {
        &self.name
    }
    fn is_folder(&self) -> bool {
        self.kind == ItemKind::Folder
    }
}

impl Named for Placed {
    fn name(&self) -> &str {
        &self.item.name
    }
    fn is_folder(&self) -> bool {
        self.item.is_folder()
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
/// the listings came in. A closer match wins over a looser one: an exact name first, then (with
/// [`Alignment::normalization`]) the same name in another normalization form, then (when matching
/// is case-insensitive) a name that differs in case too, then (with [`Alignment::extensions`]) a
/// file with the same name before its extension. So on a case-sensitive side holding `a.txt` and
/// `A.txt`, `a.txt` pairs with `a.txt`. A file and a folder with the same name pair up; telling
/// them apart is the status model's job.
///
/// *Unspecified*, decided here: the extension switch pairs files only, and only two files that
/// each have an extension (`a.txt` with `a.csv`, not `a` with `a.txt`, nor `.bashrc` with
/// `.profile`); the extension is what follows the last period. The normalization
/// switch applies to folders too, so an NFD folder pairs with its NFC twin and their contents
/// align.
pub fn align(left: Vec<Item>, right: Vec<Item>, alignment: Alignment) -> Vec<Aligned> {
    align_by_name(left, right, alignment)
}

/// Flattened alignment: pairs the files of two whole trees by name, whatever folder each sits in,
/// with the same matching and closer-match preference as [`align`]. Folders are left out, since
/// only files are aligned; links and unexpanded containers count as files.
///
/// *Unspecified*: when one side has several files with one name, a file at the same path on both
/// sides pairs first (a path that matches the way names do is the same path, so when matching is
/// case-insensitive a path that differs only in case is the same path). The others pair in folder
/// order (`a/x.txt` before `b/x.txt`) with the other side's remaining files of that name, and the
/// rest are orphans of their side. The rows don't depend on the order the items came in.
pub fn align_flat(mut left: Vec<Placed>, mut right: Vec<Placed>, alignment: Alignment) -> Vec<Aligned<Placed>> {
    for side in [&mut left, &mut right] {
        side.retain(|placed| placed.item.kind != ItemKind::Folder);
        // The name sort in `align_by_name` is stable, so files of one name stay in folder order.
        side.sort_by(|a, b| a.folder.cmp(&b.folder));
    }
    align_by_name(left, right, alignment)
}

/// What one matching pass ignores. Each pass ignores what the one before it did, and more.
#[derive(Clone, Copy, Default)]
struct Fold {
    normalization: bool,
    case: bool,
    extension: bool,
}

/// Precomposes a name (NFC), so its NFD twin gets the same key.
const NFC: ComposingNormalizerBorrowed<'static> = ComposingNormalizerBorrowed::new_nfc();

impl Fold {
    /// `None` when this pass doesn't match the item at all.
    fn key(self, name: &str, is_folder: bool) -> Option<String> {
        let name = if self.extension {
            // `rsplit_once` rather than `Path::file_stem`, so a name is split the same way on
            // every OS whatever characters it holds.
            name.rsplit_once('.').filter(|(stem, _)| !is_folder && !stem.is_empty())?.0
        } else {
            name
        };
        let name = if self.normalization { NFC.normalize(name) } else { Cow::Borrowed(name) };
        // ponytail: `to_lowercase` is Unicode lower-casing, as in `MemorySource`, not the OS's own
        // case table.
        Some(if self.case { name.to_lowercase() } else { name.into_owned() })
    }

    /// A flattened file's folder, folded the same way except for the extension.
    fn folder(self, folder: &Path) -> String {
        Self { extension: false, ..self }.key(&folder.to_string_lossy(), true).unwrap_or_default()
    }
}

fn align_by_name<T: Named>(mut left: Vec<T>, mut right: Vec<T>, alignment: Alignment) -> Vec<Aligned<T>> {
    // Sorted first, so which of two variants pairs doesn't depend on listing order.
    left.sort_by(|a, b| a.name().cmp(b.name()));
    right.sort_by(|a, b| a.name().cmp(b.name()));
    let mut left: Vec<Option<T>> = left.into_iter().map(Some).collect();
    let mut right: Vec<Option<T>> = right.into_iter().map(Some).collect();
    let mut fold = Fold::default();
    let mut folds = vec![fold];
    if alignment.normalization {
        fold.normalization = true;
        folds.push(fold);
    }
    if !alignment.case_sensitive {
        fold.case = true;
        folds.push(fold);
    }
    if alignment.extensions {
        fold.extension = true;
        folds.push(fold);
    }
    let mut rows = Vec::new();
    // Same folder first: when flattened it keeps a file that didn't move paired with itself, and
    // within one folder (no folder) the second round finds nothing new.
    for same_folder in [true, false] {
        for &fold in &folds {
            pair_by(&mut left, &mut right, &mut rows, |item| {
                Some((item.folder().filter(|_| same_folder).map(|folder| fold.folder(folder)), fold.key(item.name(), item.is_folder())?))
            });
        }
    }
    rows.extend(left.into_iter().flatten().map(|item| Aligned { name: item.name().to_owned(), left: Some(item), right: None }));
    rows.extend(right.into_iter().flatten().map(|item| Aligned { name: item.name().to_owned(), left: None, right: Some(item) }));
    rows.sort_by(|a, b| a.name.cmp(&b.name));
    rows
}

/// Moves every left item whose key matches a still-unpaired right item's into `rows`, each right
/// item pairing at most once. An item without a key pairs with nothing in this pass.
fn pair_by<T: Named, K: Eq + Hash>(left: &mut [Option<T>], right: &mut [Option<T>], rows: &mut Vec<Aligned<T>>, key: impl Fn(&T) -> Option<K>) {
    let mut by_key: HashMap<K, Vec<usize>> = HashMap::new();
    // Pushed in reverse, so `pop` hands out the first of several right items with one key.
    for (index, item) in right.iter().enumerate().rev() {
        if let Some(item_key) = item.as_ref().and_then(&key) {
            by_key.entry(item_key).or_default().push(index);
        }
    }
    for slot in left.iter_mut() {
        let Some(name_key) = slot.as_ref().and_then(&key) else { continue };
        let Some(partner) = by_key.get_mut(&name_key).and_then(Vec::pop).and_then(|index| right.get_mut(index)).and_then(Option::take) else {
            continue;
        };
        if let Some(item) = slot.take() {
            rows.push(Aligned { name: item.name().to_owned(), left: Some(item), right: Some(partner) });
        }
    }
}
