//! Local paths as people write them (`~/…`, `smb://server/share/…`, `\\server\share\…`), and what
//! the volume under a local root can do: read-only media, case-sensitive names and timestamp
//! resolution.
//!
//! The OS mounts network shares; this module only finds where. A UNC path needs nothing here:
//! Windows opens `\\server\share\…` itself.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::memory::segments;
use crate::{Capabilities, ErrorKind, SourceError};

/// The OS path for a local path as written: `~` and `~/…` start at the home folder, and
/// `smb://[user@]server/share/…` at the share's mount point (its UNC path on Windows). Anything
/// else is already an OS path.
pub fn resolve(spec: &str) -> Result<PathBuf, SourceError> {
    if let Some(rest) = spec.strip_prefix('~') {
        let after_separator = rest.strip_prefix('/').or_else(|| rest.strip_prefix('\\').filter(|_| cfg!(windows)));
        let rest = match after_separator {
            Some(rest) => rest,
            None if rest.is_empty() => "",
            None => return Err(SourceError::new(ErrorKind::Unsupported, format!("{spec}: only ~ and ~/… are expanded, not another user's ~name"))),
        };
        let home = std::env::home_dir().ok_or_else(|| SourceError::new(ErrorKind::NotFound, "this account has no home folder"))?;
        return Ok(home.join(rest));
    }
    // URL schemes ignore case.
    let Some(url) = spec.get(..6).filter(|scheme| scheme.eq_ignore_ascii_case("smb://")).map(|_| &spec[6..]) else { return Ok(PathBuf::from(spec)) };
    let (authority, path) = url.split_once('/').unwrap_or((url, ""));
    let server = authority.rsplit_once('@').map_or(authority, |(_, server)| server);
    let (share, rest) = path.split_once('/').unwrap_or((path, ""));
    let share = percent_decoded(share);
    if server.is_empty() || share.is_empty() {
        return Err(SourceError::new(ErrorKind::NotFound, format!("smb://{server}/{share} doesn't name a server and a share")));
    }
    let mut full = mount_point(server, &share)
        .ok_or_else(|| SourceError::new(ErrorKind::NotFound, format!("smb://{server}/{share} isn't mounted; connect to it in the OS first")))?;
    full.extend(segments(Path::new(&percent_decoded(rest)))?);
    Ok(full)
}

/// The value of an ASCII hex digit, which the caller has checked.
fn digit(d: u8) -> u8 {
    (d as char).to_digit(16).unwrap_or(0) as u8
}

/// `%20` and the like, as URLs and macOS mount sources write them. A `%` not followed by two hex
/// digits stays as it is.
fn percent_decoded(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        // `from_str_radix` alone would take a sign (`%+1`).
        let hex = bytes.get(i + 1..i + 3).filter(|h| h.iter().all(u8::is_ascii_hexdigit)).map(|h| h.iter().fold(0, |n, &d| n * 16 + digit(d)));
        match (bytes[i], hex) {
            (b'%', Some(byte)) => {
                out.push(byte);
                i += 3;
            }
            (byte, _) => {
                out.push(byte);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Whether a mount source (`//[user[;domain]@]server/share`, as macOS and Linux write an SMB
/// mount's) is `server`'s `share`. SMB server and share names ignore case.
///
/// ponytail: the server must be spelled as the mount spells it; an IP address or a Bonjour alias
/// (`nas._smb._tcp.local` for `nas.local`) doesn't match. Resolve names if users trip on it.
#[cfg(any(target_os = "macos", target_os = "linux", test))]
fn is_share(source: &str, server: &str, share: &str) -> bool {
    let Some((authority, path)) = source.strip_prefix("//").and_then(|s| s.split_once('/')) else { return false };
    let host = authority.rsplit_once('@').map_or(authority, |(_, host)| host);
    host.eq_ignore_ascii_case(server) && percent_decoded(path.trim_end_matches('/')).eq_ignore_ascii_case(share)
}

#[cfg(windows)]
fn mount_point(server: &str, share: &str) -> Option<PathBuf> {
    Some(PathBuf::from(format!(r"\\{server}\{share}")))
}

/// Finder and `mount_smbfs` mount shares under `/Volumes`.
///
/// ponytail: a share mounted anywhere else isn't found; read the whole mount table
/// (`getfsstat`) if that comes up.
#[cfg(target_os = "macos")]
fn mount_point(server: &str, share: &str) -> Option<PathBuf> {
    fs::read_dir("/Volumes").ok()?.flatten().map(|entry| entry.path()).find(|path| {
        rustix::fs::statfs(path).is_ok_and(|info| text(&info.f_fstypename) == "smbfs" && is_share(&text(&info.f_mntfromname), server, share))
    })
}

/// A kernel mount (`mount -t cifs`), or one GNOME's file manager made under
/// `$XDG_RUNTIME_DIR/gvfs`.
#[cfg(target_os = "linux")]
fn mount_point(server: &str, share: &str) -> Option<PathBuf> {
    let mounts = fs::read_to_string("/proc/self/mounts").unwrap_or_default();
    let kernel = mounts.lines().find_map(|line| {
        let mut fields = line.split(' ');
        let (source, point, kind) = (fields.next()?, fields.next()?, fields.next()?);
        (matches!(kind, "cifs" | "smb3") && is_share(&unescaped(source), server, share)).then(|| PathBuf::from(unescaped(point)))
    });
    kernel.or_else(|| {
        let runtime = std::env::var_os("XDG_RUNTIME_DIR").map(PathBuf::from)?;
        fs::read_dir(runtime.join("gvfs")).ok()?.flatten().map(|entry| entry.path()).find(|path| {
            let name = path.file_name().and_then(|name| name.to_str()).unwrap_or_default();
            let Some(keys) = name.strip_prefix("smb-share:") else { return false };
            let value = |key: &str| keys.split(',').find_map(|pair| pair.strip_prefix(key)?.strip_prefix('=')).map(percent_decoded);
            value("server").is_some_and(|s| s.eq_ignore_ascii_case(server)) && value("share").is_some_and(|s| s.eq_ignore_ascii_case(share))
        })
    })
}

/// `/proc/self/mounts` writes a space as `\040`, and a tab, newline or backslash the same way.
#[cfg(any(target_os = "linux", test))]
fn unescaped(field: &str) -> String {
    let bytes = field.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        // Three octal digits; `\400` and up isn't a byte, so it stays as written.
        let octal = bytes
            .get(i + 1..i + 4)
            .filter(|o| o.iter().all(|d| (b'0'..=b'7').contains(d)) && o[0] <= b'3')
            .map(|o| o.iter().fold(0, |n, &d| n * 8 + digit(d)));
        match (bytes[i], octal) {
            (b'\\', Some(byte)) => {
                out.push(byte);
                i += 4;
            }
            (byte, _) => {
                out.push(byte);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
fn mount_point(_: &str, _: &str) -> Option<PathBuf> {
    None
}

/// What the volume holding `root` can do. What can't be found out keeps the OS's usual default:
/// writable, exact times, and case-insensitive names only on Windows and macOS.
pub(crate) fn capabilities(root: &Path) -> Capabilities {
    let (file_system, read_only) = file_system(root).unwrap_or_default();
    Capabilities {
        read_only,
        case_sensitive: case_sensitive(root, &file_system),
        recycle_bin: false,
        timestamp_resolution: resolution(&file_system),
        windows_attributes: cfg!(windows),
        // FAT and exFAT keep no Unix modes: macOS and Linux accept a `chmod` there and change
        // nothing.
        //
        // ponytail: other volumes that ignore modes the same way (an SMB share mounted without
        // Unix extensions, say) still count as keeping them; name them if that comes up.
        unix_permissions: cfg!(unix) && !fat(&file_system),
    }
}

/// FAT and exFAT, by the name macOS or Windows gives them (Linux names come from [`file_system`]).
fn fat(file_system: &str) -> bool {
    matches!(file_system.to_ascii_lowercase().as_str(), "msdos" | "fat" | "fat32" | "exfat")
}

/// The step a file system stores modified times in, by the name macOS or Windows gives it (Linux
/// names come from [`file_system`]).
fn resolution(file_system: &str) -> Duration {
    match file_system.to_ascii_lowercase().as_str() {
        "msdos" | "fat" | "fat32" => Duration::from_secs(2),
        "hfs" => Duration::from_secs(1),
        "exfat" => Duration::from_millis(10),
        "ntfs" | "refs" => Duration::from_nanos(100),
        _ => Duration::ZERO,
    }
}

/// The file-system name and whether the volume is mounted read-only.
#[cfg(target_os = "macos")]
fn file_system(root: &Path) -> Option<(String, bool)> {
    let name = text(&rustix::fs::statfs(root).ok()?.f_fstypename);
    Some((name, read_only_mount(root)))
}

/// A NUL-terminated name from a `statfs` record.
#[cfg(target_os = "macos")]
fn text(chars: &[std::ffi::c_char]) -> String {
    let bytes: Vec<u8> = chars.iter().take_while(|&&c| c != 0).map(|&c| c as u8).collect();
    String::from_utf8_lossy(&bytes).into_owned()
}

#[cfg(target_os = "linux")]
fn file_system(root: &Path) -> Option<(String, bool)> {
    // `MSDOS_SUPER_MAGIC` and `EXFAT_SUPER_MAGIC` (`<linux/magic.h>`).
    let name = match rustix::fs::statfs(root).ok()?.f_type as u32 {
        0x4d44 => "msdos",
        0x2011_bab0 => "exfat",
        _ => "",
    };
    Some((name.to_owned(), read_only_mount(root)))
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn read_only_mount(root: &Path) -> bool {
    rustix::fs::statvfs(root).is_ok_and(|info| info.f_flag.contains(rustix::fs::StatVfsMountFlags::RDONLY))
}

#[cfg(windows)]
fn file_system(root: &Path) -> Option<(String, bool)> {
    /// `FILE_READ_ONLY_VOLUME`, from `winnt.h`.
    const READ_ONLY_VOLUME: u32 = 0x0008_0000;
    let (name, flags) = windows::volume(root)?;
    Some((name, flags & READ_ONLY_VOLUME != 0))
}

#[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
fn file_system(_: &Path) -> Option<(String, bool)> {
    None
}

/// Whether names in `root` differ by case. macOS answers for the volume; FAT and exFAT ignore case
/// everywhere; otherwise a name in `root` is tried with its case flipped, and an empty `root`
/// gets the OS's usual default (case-insensitive only on Windows and macOS).
///
/// ponytail: on Linux and Windows an empty root of a case-insensitive file system other than FAT
/// (a CIFS mount, an ext4 casefold folder) gets the default, and the answer is kept for the
/// source's life. Ask the volume there too if that comes up.
fn case_sensitive(root: &Path, file_system: &str) -> bool {
    #[cfg(target_os = "macos")]
    if let Some(answer) = macos::case_sensitive(root) {
        return answer;
    }
    if fat(file_system) {
        return false;
    }
    probe_case(root).unwrap_or(cfg!(not(any(windows, target_os = "macos"))))
}

/// Tries a name in `root` with its ASCII letters' case flipped. `None` when `root` holds no
/// such name to try: probing `root`'s own name would ask the volume above a mount point.
///
/// ponytail: one probe answers for the whole volume; Windows can switch case sensitivity per
/// folder, and a probe per folder can follow if that matters.
fn probe_case(root: &Path) -> Option<bool> {
    let names: Vec<String> = fs::read_dir(root).ok()?.flatten().filter_map(|entry| entry.file_name().into_string().ok()).collect();
    for name in &names {
        let flipped: String = name.chars().map(|c| if c.is_ascii_lowercase() { c.to_ascii_uppercase() } else { c.to_ascii_lowercase() }).collect();
        if flipped == *name {
            continue;
        }
        // Both spellings side by side is an answer too.
        if names.contains(&flipped) {
            return Some(true);
        }
        match fs::symlink_metadata(root.join(&flipped)) {
            Ok(_) => return Some(false),
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Some(true),
            Err(_) => {}
        }
    }
    None
}

#[cfg(target_os = "macos")]
#[allow(unsafe_code)] // AGENTS.md "## This repository", rule 13, names this module.
mod macos {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;
    use std::path::Path;

    /// Whether the volume holding `path` keeps names that differ only by case apart
    /// (`_PC_CASE_SENSITIVE`). `None` when macOS can't say.
    pub(super) fn case_sensitive(path: &Path) -> Option<bool> {
        let path = CString::new(path.as_os_str().as_bytes()).ok()?;
        // SAFETY: `path` is a NUL-terminated string that lives for the whole call, and
        // `pathconf` only reads it.
        let answer = unsafe { libc::pathconf(path.as_ptr(), libc::_PC_CASE_SENSITIVE) };
        (answer >= 0).then_some(answer == 1)
    }
}

#[cfg(windows)]
#[allow(unsafe_code)] // AGENTS.md "## This repository", rule 13, names this module.
mod windows {
    use std::os::windows::fs::OpenOptionsExt;
    use std::os::windows::io::AsRawHandle;
    use std::path::Path;
    use std::ptr::null_mut;

    use windows_sys::Win32::Storage::FileSystem::{FILE_FLAG_BACKUP_SEMANTICS, GetVolumeInformationByHandleW};

    /// The file-system name (`NTFS`, `FAT32`, …) and flags of the volume holding `path`.
    pub(super) fn volume(path: &Path) -> Option<(String, u32)> {
        // Backup semantics lets a folder be opened; read access locks nothing.
        let folder = std::fs::OpenOptions::new().read(true).custom_flags(FILE_FLAG_BACKUP_SEMANTICS).open(path).ok()?;
        let mut flags = 0u32;
        // MAX_PATH + 1, as the documentation asks.
        let mut name = [0u16; 261];
        // SAFETY: `folder` keeps the handle open for the whole call. `flags` and `name` are live
        // locals, and `name`'s length is passed with it. The outputs left null have zero sizes or
        // are documented as optional.
        let ok = unsafe {
            GetVolumeInformationByHandleW(
                folder.as_raw_handle(),
                null_mut(),
                0,
                null_mut(),
                null_mut(),
                &mut flags,
                name.as_mut_ptr(),
                name.len() as u32,
            )
        };
        if ok == 0 {
            return None;
        }
        let len = name.iter().position(|&c| c == 0).unwrap_or(name.len());
        Some((String::from_utf16_lossy(&name[..len]), flags))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_mount_source_matches_its_share_whatever_the_user_case_or_encoding() {
        assert!(is_share("//alex@NAS/My%20Files", "nas", "my files"));
        assert!(is_share("//nas/files/", "nas", "files"));
        assert!(!is_share("//nas/files/sub", "nas", "files"), "a mount of a subfolder isn't the share");
        assert!(!is_share("//other/files", "nas", "files"));
        assert!(!is_share("/dev/disk3s1", "nas", "files"));
    }

    #[test]
    fn mount_table_escapes_are_undone() {
        assert_eq!(unescaped(r"/mnt/my\040share\134x"), r"/mnt/my share\x");
        assert_eq!(unescaped(r"/mnt/odd\9"), r"/mnt/odd\9");
        assert_eq!(unescaped(r"/mnt/\+12\400"), r"/mnt/\+12\400", "a sign or a value past a byte isn't an escape");
    }

    #[test]
    fn only_a_percent_and_two_hex_digits_is_an_escape() {
        assert_eq!(percent_decoded("My%20Files%2b%+1%4"), "My Files+%+1%4");
    }

    #[test]
    fn a_tilde_path_starts_at_the_home_folder() {
        let home = std::env::home_dir().unwrap();
        assert_eq!(resolve("~").unwrap(), home);
        assert_eq!(resolve("~/docs/a.txt").unwrap(), home.join("docs/a.txt"));
        assert_eq!(resolve("~bob/x").unwrap_err().kind, ErrorKind::Unsupported);
    }

    #[test]
    fn an_smb_url_is_its_unc_path_on_windows_and_elsewhere_must_be_mounted() {
        let resolved = resolve("smb://alex@nas.invalid/My%20Share/a/b.txt");
        if cfg!(windows) {
            // Windows opens the share itself, and finds out then whether it's there.
            assert_eq!(resolved.unwrap(), PathBuf::from(r"\\nas.invalid\My Share\a\b.txt"));
        } else {
            let error = resolved.unwrap_err();
            assert!(error.kind == ErrorKind::NotFound && error.message.contains("isn't mounted"), "{error:?}");
        }
    }

    #[test]
    fn an_smb_url_error_leaves_out_the_password() {
        let error = resolve("SMB://alex:hunter2@nas.invalid").unwrap_err();
        assert!(error.kind == ErrorKind::NotFound && !error.message.contains("hunter2"), "{error:?}");
    }

    /// People type these paths, and the Linux mount table is read as text, so neither parser
    /// panics or hangs, whatever the bytes. They are private and `LocalSource::open` reads the
    /// volume each time, so the sweep sits here rather than in `tests/it`.
    #[test]
    fn a_damaged_path_or_mount_line_never_panics_or_hangs() {
        use craft_testkit::hostile::{HammerConfig, hammer};

        hammer(&HammerConfig::new("smb and ~ paths", 2_000), b"smb://al%20ex;dom@nas/My%2xShare/a%2Fb/~c", |data| {
            let spec = String::from_utf8_lossy(data);
            let _ = resolve(&spec);
            let _ = resolve(&format!("~/{spec}"));
        });
        hammer(&HammerConfig::new("mount table fields", 2_000), br"//u;d@nas/my\040share\134x /mnt/\0111", |data| {
            let field = String::from_utf8_lossy(data);
            let _ = is_share(&unescaped(&field), "nas", "my share");
        });
    }
}
