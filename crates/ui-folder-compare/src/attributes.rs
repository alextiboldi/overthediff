//! The Attributes, Owner and Group columns (E08-S6-T4): what an item's source reports, in the
//! form its platform shows it. The form follows the item's source, not the OS the app runs on, so
//! a Windows share's items show attribute codes on a Mac too.
//!
//! - **Attributes** shows Windows attribute codes as letters, or a Unix mode string as `ls -l`
//!   shows it (`-rw-r--r--`, `drwxr-xr-x`, setuid `s`, sticky `t`).
//! - **Owner** and **Group** show the Unix owner and group names, blank for a Windows item.
//!
//! *Unspecified*, decided here: the codes show in alphabetical order with no gaps (`HR` for a
//! read-only hidden file), and a mode with no file-type bits takes its type from the item's kind.

use otd_source::{Attributes, Item, ItemKind};

/// Each Windows code's letter and its `FILE_ATTRIBUTE_*` bit, in display order.
const CODES: &[(char, u32)] = &[
    ('A', 0x20),      // ARCHIVE
    ('C', 0x800),     // COMPRESSED
    ('D', 0x10),      // DIRECTORY
    ('E', 0x4000),    // ENCRYPTED
    ('H', 0x2),       // HIDDEN
    ('I', 0x2000),    // NOT_CONTENT_INDEXED
    ('L', 0x400),     // REPARSE_POINT (symlinks among them)
    ('M', 0x40_0000), // RECALL_ON_DATA_ACCESS
    ('N', 0x80),      // NORMAL
    ('O', 0x1000),    // OFFLINE
    ('P', 0x8_0000),  // PINNED
    ('R', 0x1),       // READONLY
    ('S', 0x4),       // SYSTEM
    ('T', 0x100),     // TEMPORARY
    ('U', 0x10_0000), // UNPINNED
    ('Z', 0x200),     // SPARSE_FILE
];

pub(crate) fn attributes(item: &Item) -> Option<String> {
    match item.attributes.as_ref()? {
        Attributes::Windows(codes) => Some(CODES.iter().filter(|(_, bit)| codes & bit != 0).map(|(letter, _)| letter).collect()),
        Attributes::Unix { mode, .. } => Some(mode_string(*mode, item.kind)),
    }
}

pub(crate) fn owner(item: &Item) -> Option<String> {
    match item.attributes.as_ref()? {
        Attributes::Unix { owner, .. } if !owner.is_empty() => Some(owner.clone()),
        _ => None,
    }
}

pub(crate) fn group(item: &Item) -> Option<String> {
    match item.attributes.as_ref()? {
        Attributes::Unix { group, .. } if !group.is_empty() => Some(group.clone()),
        _ => None,
    }
}

/// `mode` as `ls -l` shows it: the type, then read, write and execute for owner, group and others.
fn mode_string(mode: u32, kind: ItemKind) -> String {
    let kind = match mode & 0o170_000 {
        0o140_000 => 's',
        0o120_000 => 'l',
        0o100_000 => '-',
        0o060_000 => 'b',
        0o040_000 => 'd',
        0o020_000 => 'c',
        0o010_000 => 'p',
        _ => match kind {
            ItemKind::File => '-',
            ItemKind::Folder => 'd',
            ItemKind::Link => 'l',
        },
    };
    let mut text = String::from(kind);
    // Owner, group, others: their read bit, and the special bit that shares their execute place.
    for (shift, special, set) in [(6, 0o4000, 's'), (3, 0o2000, 's'), (0, 0o1000, 't')] {
        let bits = mode >> shift;
        text.push(if bits & 0o4 != 0 { 'r' } else { '-' });
        text.push(if bits & 0o2 != 0 { 'w' } else { '-' });
        text.push(match (bits & 0o1 != 0, mode & special != 0) {
            (true, true) => set,
            (false, true) => set.to_ascii_uppercase(),
            (true, false) => 'x',
            (false, false) => '-',
        });
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modes_read_as_ls_shows_them_special_bits_included() {
        for (mode, kind, shown) in [
            (0o100_644, ItemKind::File, "-rw-r--r--"),
            (0o040_755, ItemKind::Folder, "drwxr-xr-x"),
            (0o120_777, ItemKind::Link, "lrwxrwxrwx"),
            (0o104_755, ItemKind::File, "-rwsr-xr-x"),
            (0o102_644, ItemKind::File, "-rw-r-Sr--"),
            (0o041_777, ItemKind::Folder, "drwxrwxrwt"),
            (0o041_776, ItemKind::Folder, "drwxrwxrwT"),
            // No type bits, as a source that keeps only permissions reports: the kind gives it.
            (0o750, ItemKind::Folder, "drwxr-x---"),
        ] {
            assert_eq!(mode_string(mode, kind), shown, "{mode:o}");
        }
    }

    #[test]
    fn windows_codes_show_exactly_the_letters_of_the_bits_set() {
        let item = |codes| Item {
            name: "a".into(),
            kind: ItemKind::File,
            size: 0,
            modified: None,
            created: None,
            attributes: Some(Attributes::Windows(codes)),
            link_target: None,
            crc32: None,
            container: false,
            error: None,
        };
        let letters = |codes| attributes(&item(codes)).unwrap_or_default().chars().collect::<std::collections::BTreeSet<_>>();
        // Read-only and hidden (the AC), then codes from both ends of the bit range.
        assert_eq!(letters(0x1 | 0x2), ['H', 'R'].into());
        assert_eq!(letters(0x10 | 0x20 | 0x400 | 0x40_0000), ['A', 'D', 'L', 'M'].into());
        assert!(letters(0).is_empty());
        assert_eq!(owner(&item(0x1)), None, "a Windows item has no Unix owner");
    }
}
