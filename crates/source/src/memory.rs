//! The in-memory reference source. Tests declare trees in it with sizes, times, links, unreadable
//! items, source-provided CRCs and expandable containers, and the contract suite uses it to
//! describe each case's fixture.

use std::collections::VecDeque;
use std::io::{Cursor, Read};
use std::path::{Component, Path};
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::{Duration, SystemTime};

use crate::{Attributes, Capabilities, DataSource, Delete, ErrorKind, Item, ItemKind, Links, Permissions, SourceError};

/// More links than this on one path is a loop, as on most OSes.
const MAX_LINK_HOPS: usize = 40;

/// A data source held in memory. The builder methods (`file`, `folder`, `link`, …) declare a
/// fixture and ignore [`Capabilities::read_only`]; the [`DataSource`] methods honour it.
///
/// Builder paths use `/` between names. Missing parent folders are created, and an item already
/// at a path is replaced.
#[derive(Debug)]
pub struct MemorySource {
    tree: Mutex<Tree>,
}

#[derive(Debug, Clone)]
struct Tree {
    caps: Capabilities,
    root: Node,
}

#[derive(Debug, Clone)]
struct Node {
    kind: ItemKind,
    content: Vec<u8>,
    // ponytail: a Vec with linear lookup; a map if fixtures ever grow past a few hundred items.
    children: Vec<(String, Node)>,
    container: bool,
    link_target: Option<String>,
    modified: SystemTime,
    attributes: Option<Attributes>,
    crc32: Option<u32>,
    unreadable: bool,
}

impl Node {
    fn new(kind: ItemKind, modified: SystemTime) -> Self {
        Self {
            kind,
            content: Vec::new(),
            children: Vec::new(),
            container: false,
            link_target: None,
            modified,
            attributes: None,
            crc32: None,
            unreadable: false,
        }
    }

    fn listable(&self) -> bool {
        self.kind == ItemKind::Folder || self.container
    }
}

impl Clone for MemorySource {
    fn clone(&self) -> Self {
        Self { tree: Mutex::new(self.lock().clone()) }
    }
}

impl MemorySource {
    pub fn new(capabilities: Capabilities) -> Self {
        Self { tree: Mutex::new(Tree { caps: capabilities, root: Node::new(ItemKind::Folder, SystemTime::UNIX_EPOCH) }) }
    }

    /// The same tree, reporting and honouring other capabilities.
    #[must_use]
    pub fn with_capabilities(mut self, capabilities: Capabilities) -> Self {
        self.tree_mut().caps = capabilities;
        self
    }

    #[must_use]
    pub fn file(mut self, path: &str, content: impl Into<Vec<u8>>, modified: SystemTime) -> Self {
        self.put(path, Node { content: content.into(), ..Node::new(ItemKind::File, modified) });
        self
    }

    #[must_use]
    pub fn folder(mut self, path: &str, modified: SystemTime) -> Self {
        self.put(path, Node::new(ItemKind::Folder, modified));
        self
    }

    /// A link to `target`, which is resolved from the link's folder and may use `..`.
    #[must_use]
    pub fn link(mut self, path: &str, target: &str, modified: SystemTime) -> Self {
        self.put(path, Node { link_target: Some(target.to_owned()), ..Node::new(ItemKind::Link, modified) });
        self
    }

    /// A file whose children list like a folder's. Declare them with paths under it.
    #[must_use]
    pub fn container(mut self, path: &str, content: impl Into<Vec<u8>>, modified: SystemTime) -> Self {
        self.put(path, Node { content: content.into(), container: true, ..Node::new(ItemKind::File, modified) });
        self
    }

    /// Marks an item unreadable: it lists with an access-denied error, and its content or
    /// children can't be read.
    #[must_use]
    pub fn unreadable(mut self, path: &str) -> Self {
        self.edit(path, |node| node.unreadable = true);
        self
    }

    /// Gives a file a source-provided CRC-32.
    #[must_use]
    pub fn crc32(mut self, path: &str, crc: u32) -> Self {
        self.edit(path, |node| node.crc32 = Some(crc));
        self
    }

    #[must_use]
    pub fn attributes(mut self, path: &str, attributes: Attributes) -> Self {
        self.edit(path, |node| node.attributes = Some(attributes));
        self
    }

    fn lock(&self) -> MutexGuard<'_, Tree> {
        self.tree.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn tree_mut(&mut self) -> &mut Tree {
        self.tree.get_mut().unwrap_or_else(PoisonError::into_inner)
    }

    fn put(&mut self, path: &str, node: Node) {
        let tree = self.tree_mut();
        let case_sensitive = tree.caps.case_sensitive;
        let names: Vec<&str> = path.split('/').filter(|name| !name.is_empty()).collect();
        let Some((last, parents)) = names.split_last() else { return };
        let mut folder = &mut tree.root;
        for name in parents {
            let found = find(&folder.children, name, case_sensitive);
            let index = match found {
                Some(i) if folder.children[i].1.listable() => i,
                _ => {
                    let new = ((*name).to_owned(), Node::new(ItemKind::Folder, node.modified));
                    set_child(&mut folder.children, found, new)
                }
            };
            folder = &mut folder.children[index].1;
        }
        let found = find(&folder.children, last, case_sensitive);
        set_child(&mut folder.children, found, ((*last).to_owned(), node));
    }

    fn edit(&mut self, path: &str, change: impl FnOnce(&mut Node)) {
        let tree = self.tree_mut();
        let case_sensitive = tree.caps.case_sensitive;
        let mut node = &mut tree.root;
        for name in path.split('/').filter(|name| !name.is_empty()) {
            let Some(i) = find(&node.children, name, case_sensitive) else { return };
            node = &mut node.children[i].1;
        }
        change(node);
    }
}

/// Replaces the child at `found`, or appends it, and returns its index.
fn set_child(children: &mut Vec<(String, Node)>, found: Option<usize>, child: (String, Node)) -> usize {
    match found {
        Some(i) => {
            children[i] = child;
            i
        }
        None => {
            children.push(child);
            children.len() - 1
        }
    }
}

// ponytail: `to_lowercase` is Unicode lower-casing, not the OS's own case table.
fn find(children: &[(String, Node)], name: &str, case_sensitive: bool) -> Option<usize> {
    children.iter().position(|(n, _)| n == name || (!case_sensitive && n.to_lowercase() == name.to_lowercase()))
}

fn error(kind: ErrorKind, message: String) -> SourceError {
    SourceError { kind, message }
}

fn not_found(name: &str) -> SourceError {
    error(ErrorKind::NotFound, format!("{name} doesn't exist"))
}

/// A contract path as names. `..`, a root or a drive prefix would leave the source, so they are
/// not found.
pub(crate) fn segments(path: &Path) -> Result<Vec<String>, SourceError> {
    let outside = || error(ErrorKind::NotFound, format!("{} is outside the source", path.display()));
    let mut names = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(name) => names.push(name.to_str().ok_or_else(outside)?.to_owned()),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => return Err(outside()),
        }
    }
    Ok(names)
}

impl Tree {
    /// The node at stored names, as [`Tree::resolve`] returns them.
    fn node(&self, at: &[String]) -> Result<&Node, SourceError> {
        let mut node = &self.root;
        for name in at {
            node = &node.children.iter().find(|(n, _)| n == name).ok_or_else(|| not_found(name))?.1;
        }
        Ok(node)
    }

    fn node_mut(&mut self, at: &[String]) -> Result<&mut Node, SourceError> {
        let mut node = &mut self.root;
        for name in at {
            node = &mut node.children.iter_mut().find(|(n, _)| n == name).ok_or_else(|| not_found(name))?.1;
        }
        Ok(node)
    }

    /// Turns names into stored names, following links in every folder on the way, and in the last
    /// name too when `follow_last` is set.
    fn resolve(&self, names: &[String], follow_last: bool) -> Result<Vec<String>, SourceError> {
        let mut pending: VecDeque<String> = names.iter().cloned().collect();
        let mut done: Vec<String> = Vec::new();
        let mut hops = 0;
        while let Some(name) = pending.pop_front() {
            let parent = self.node(&done)?;
            if !parent.listable() {
                return Err(error(ErrorKind::NotFound, format!("{name} isn't inside a folder")));
            }
            if parent.unreadable {
                return Err(error(ErrorKind::AccessDenied, format!("access denied reading the folder that holds {name}")));
            }
            let (stored, child) =
                find(&parent.children, &name, self.caps.case_sensitive).and_then(|i| parent.children.get(i)).ok_or_else(|| not_found(&name))?;
            match &child.link_target {
                Some(target) if follow_last || !pending.is_empty() => {
                    hops += 1;
                    if hops > MAX_LINK_HOPS {
                        return Err(error(ErrorKind::Io, format!("too many levels of links at {name}")));
                    }
                    let mut next = done.clone();
                    for component in Path::new(target).components() {
                        match component {
                            Component::Normal(part) => next.push(part.to_string_lossy().into_owned()),
                            Component::CurDir => {}
                            Component::ParentDir if next.pop().is_some() => {}
                            _ => return Err(error(ErrorKind::NotFound, format!("{name} points outside the source"))),
                        }
                    }
                    next.extend(pending.drain(..));
                    pending = next.into();
                    done.clear();
                }
                _ => done.push(stored.clone()),
            }
        }
        Ok(done)
    }

    /// The node at `at` as an item, following it when it is a link and `links` says so.
    fn item(&self, at: &[String], node: &Node, links: Links) -> Item {
        let name = at.last().cloned().unwrap_or_default();
        if links == Links::Follow && node.link_target.is_some() {
            return match self.resolve(at, true).and_then(|target| self.node(&target)) {
                Ok(target) => Item { link_target: node.link_target.clone(), ..self.report(name, target) },
                Err(e) => Item { error: Some(e), ..self.report(name, node) },
            };
        }
        self.report(name, node)
    }

    fn report(&self, name: String, node: &Node) -> Item {
        let error = node.unreadable.then(|| error(ErrorKind::AccessDenied, format!("access denied reading {name}")));
        Item {
            size: if node.kind == ItemKind::File { node.content.len() as u64 } else { 0 },
            modified: Some(self.coarsen(node.modified)),
            created: None,
            attributes: node.attributes.clone(),
            link_target: node.link_target.clone(),
            crc32: node.crc32,
            container: node.container,
            kind: node.kind,
            name,
            error,
        }
    }

    /// Rounds a time down to the timestamp resolution, as a FAT volume stores it.
    fn coarsen(&self, time: SystemTime) -> SystemTime {
        let step = self.caps.timestamp_resolution.as_nanos();
        let Ok(since) = time.duration_since(SystemTime::UNIX_EPOCH) else { return time };
        if step == 0 {
            return time;
        }
        u64::try_from(since.as_nanos() % step).ok().and_then(|extra| time.checked_sub(Duration::from_nanos(extra))).unwrap_or(time)
    }

    fn writable(&self) -> Result<(), SourceError> {
        if self.caps.read_only { Err(error(ErrorKind::AccessDenied, "the source is read-only".to_owned())) } else { Ok(()) }
    }

    /// The stored names of the folder that holds `names`' last item, and that item's index in
    /// it, if it exists.
    fn locate(&self, names: &[String]) -> Result<(Vec<String>, Option<usize>), SourceError> {
        let Some((name, parents)) = names.split_last() else {
            return Err(error(ErrorKind::Unsupported, "the root of a source can't be replaced, renamed or deleted".to_owned()));
        };
        let parent_at = self.resolve(parents, true)?;
        let parent = self.node(&parent_at)?;
        if !parent.listable() {
            return Err(error(ErrorKind::NotFound, format!("{name} isn't inside a folder")));
        }
        if parent.unreadable {
            return Err(error(ErrorKind::AccessDenied, format!("access denied changing the folder that holds {name}")));
        }
        Ok((parent_at, find(&parent.children, name, self.caps.case_sensitive)))
    }

    /// The existing item at `names`, unfollowed, for a change to it.
    fn existing_mut(&mut self, names: &[String]) -> Result<&mut Node, SourceError> {
        if names.is_empty() {
            return Ok(&mut self.root);
        }
        let (parent_at, found) = self.locate(names)?;
        let name = names.last().map_or("", String::as_str);
        let index = found.ok_or_else(|| not_found(name))?;
        self.node_mut(&parent_at)?.children.get_mut(index).map(|(_, node)| node).ok_or_else(|| not_found(name))
    }
}

impl DataSource for MemorySource {
    fn capabilities(&self) -> Capabilities {
        self.lock().caps
    }

    fn list(&self, folder: &Path, links: Links) -> Result<Vec<Item>, SourceError> {
        let tree = self.lock();
        let at = tree.resolve(&segments(folder)?, true)?;
        let node = tree.node(&at)?;
        if !node.listable() {
            return Err(error(ErrorKind::Io, format!("{} isn't a folder", folder.display())));
        }
        if node.unreadable {
            return Err(error(ErrorKind::AccessDenied, format!("access denied listing {}", folder.display())));
        }
        Ok(node
            .children
            .iter()
            .map(|(name, child)| {
                let mut child_at = at.clone();
                child_at.push(name.clone());
                tree.item(&child_at, child, links)
            })
            .collect())
    }

    fn stat(&self, path: &Path, links: Links) -> Result<Item, SourceError> {
        let tree = self.lock();
        let at = tree.resolve(&segments(path)?, false)?;
        Ok(tree.item(&at, tree.node(&at)?, links))
    }

    fn open_read(&self, path: &Path) -> Result<Box<dyn Read + Send>, SourceError> {
        let tree = self.lock();
        let node = tree.node(&tree.resolve(&segments(path)?, true)?)?;
        if node.kind != ItemKind::File {
            return Err(error(ErrorKind::Io, format!("{} isn't a file", path.display())));
        }
        if node.unreadable {
            return Err(error(ErrorKind::AccessDenied, format!("access denied reading {}", path.display())));
        }
        Ok(Box::new(Cursor::new(node.content.clone())))
    }

    fn write(&self, path: &Path, content: &mut dyn Read) -> Result<(), SourceError> {
        self.lock().writable()?;
        let names = segments(path)?;
        // Read before locking again, so a reader that calls back into this source can't deadlock.
        let mut bytes = Vec::new();
        content.read_to_end(&mut bytes).map_err(|e| error(ErrorKind::Io, e.to_string()))?;
        let mut tree = self.lock();
        tree.writable()?;
        let (parent_at, found) = tree.locate(&names)?;
        let name = names.last().cloned().unwrap_or_default();
        let parent = tree.node_mut(&parent_at)?;
        match found.and_then(|i| parent.children.get_mut(i)) {
            Some((_, node)) if node.kind == ItemKind::Folder => Err(error(ErrorKind::Io, format!("a folder named {name} is in the way"))),
            Some((_, node)) => {
                *node = Node { content: bytes, attributes: node.attributes.take(), ..Node::new(ItemKind::File, SystemTime::now()) };
                Ok(())
            }
            None => {
                parent.children.push((name, Node { content: bytes, ..Node::new(ItemKind::File, SystemTime::now()) }));
                Ok(())
            }
        }
    }

    fn create_folder(&self, path: &Path) -> Result<(), SourceError> {
        let mut tree = self.lock();
        tree.writable()?;
        let names = segments(path)?;
        if names.is_empty() {
            // The root is always a folder.
            return Ok(());
        }
        let (parent_at, found) = tree.locate(&names)?;
        let name = names.last().cloned().unwrap_or_default();
        let parent = tree.node_mut(&parent_at)?;
        match found.and_then(|i| parent.children.get(i)) {
            Some((_, node)) if node.kind == ItemKind::Folder => Ok(()),
            Some(_) => Err(error(ErrorKind::Io, format!("a file named {name} is in the way"))),
            None => {
                parent.children.push((name, Node::new(ItemKind::Folder, SystemTime::now())));
                Ok(())
            }
        }
    }

    fn delete(&self, path: &Path, how: Delete) -> Result<(), SourceError> {
        let mut tree = self.lock();
        tree.writable()?;
        let names = segments(path)?;
        let (parent_at, found) = tree.locate(&names)?;
        let index = found.ok_or_else(|| not_found(&path.display().to_string()))?;
        if how == Delete::ToRecycleBin && !tree.caps.recycle_bin {
            return Err(error(ErrorKind::Unsupported, "the recycle bin is unavailable on this source".to_owned()));
        }
        // ponytail: the recycle bin keeps nothing, since the contract has no restore.
        tree.node_mut(&parent_at)?.children.remove(index);
        Ok(())
    }

    fn rename(&self, path: &Path, new_name: &str) -> Result<(), SourceError> {
        let mut tree = self.lock();
        tree.writable()?;
        if segments(Path::new(new_name)).ok().as_deref() != Some(&[new_name.to_owned()]) {
            return Err(error(ErrorKind::Unsupported, format!("{new_name} isn't a single name")));
        }
        let names = segments(path)?;
        let (parent_at, found) = tree.locate(&names)?;
        let index = found.ok_or_else(|| not_found(&path.display().to_string()))?;
        let case_sensitive = tree.caps.case_sensitive;
        let parent = tree.node_mut(&parent_at)?;
        if find(&parent.children, new_name, case_sensitive).is_some_and(|other| other != index) {
            return Err(error(ErrorKind::Io, format!("an item named {new_name} already exists")));
        }
        if let Some((name, _)) = parent.children.get_mut(index) {
            new_name.clone_into(name);
        }
        Ok(())
    }

    fn set_modified(&self, path: &Path, time: SystemTime) -> Result<(), SourceError> {
        let mut tree = self.lock();
        tree.writable()?;
        tree.existing_mut(&segments(path)?)?.modified = time;
        Ok(())
    }

    fn set_permissions(&self, path: &Path, permissions: Permissions) -> Result<(), SourceError> {
        let mut tree = self.lock();
        tree.writable()?;
        let supported = match permissions {
            Permissions::Windows(_) => tree.caps.windows_attributes,
            Permissions::UnixMode(_) => tree.caps.unix_permissions,
        };
        if !supported {
            return Err(error(ErrorKind::Unsupported, "this source doesn't keep those attributes".to_owned()));
        }
        let node = tree.existing_mut(&segments(path)?)?;
        node.attributes = Some(match (permissions, node.attributes.take()) {
            (Permissions::Windows(codes), _) => Attributes::Windows(codes),
            (Permissions::UnixMode(mode), Some(Attributes::Unix { owner, group, .. })) => Attributes::Unix { mode, owner, group },
            (Permissions::UnixMode(mode), _) => Attributes::Unix { mode, owner: String::new(), group: String::new() },
        });
        Ok(())
    }
}
