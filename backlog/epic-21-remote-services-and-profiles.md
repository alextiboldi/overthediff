# E21 — Remote services and profiles

**Group**: feature
**Label**: `area:remote`
**Size**: L
**Depends on**: E01, E04 (foundations); E07, E08 (views); E09 (log panel, for E21-S2)
**Source**: catalogue F-7.4, F-7.5, F-7.6, F-26.7, F-26.11, F-8.6. Spec-derived from the feature catalogue and not yet validated. Items marked *inferred* are analyst or PO inferences.

Remote services let any folder input be an FTP, SFTP or FTPS site, cloud storage (Amazon S3, Dropbox, OneDrive), a WebDAV server or a Subversion repository. Named profiles hold the connection settings. Each remote kind is a data source behind the same contract as local folders, so comparisons, filters and operations work on it unchanged.

## Behavioral contract
- **Data**:
  - *Profile*: name, kind (FTP, Amazon S3, WebDAV, Dropbox, OneDrive or Subversion), description and kind-specific settings. Secrets (passwords, keys and tokens) are stored as decided in E21-S1-T7, and only when "Save password" is on.
  - *FTP Global settings*: shared by every FTP profile. They hold the anonymous-login email, the SSH key and SSL certificate files, and the ASCII types list.
  - *`<default>` FTP profile*: settings that named FTP profiles inherit.
  - *Quick Connect*: an unsaved, ad-hoc profile.
- **Routes/endpoints**:
  - Tools > Profiles.
  - Specs: `ftp://`, `sftp://`, `ftps://`, `svn://`, `svn+http://`, `svn+https://`, `svn+ssh://`, `profile:MyProfile` and `profile:MyProfile?MyFolder/MyFile.ext`.
  - Browse for Folder > Browse Using Profile.
  - Hidden-by-default commands Open File Using Profile and Save File Using Profile As.
  - Folder Handling option Touch local files when copying to an FTP site.
  - Log category FTP trace.
  - Tweak Use IPv6.
- **Integrations**:
  - E04: each remote kind is a provider behind the data-source contract.
  - Remote services: FTP, SFTP and FTPS servers, Amazon S3, WebDAV servers, Dropbox, OneDrive and Subversion repositories.
  - The system browser, for authorisation.
  - ssh-agent and pageant.
- **Rules/states**:
  - Remote timestamps are converted to local time using the server time zone.
  - Each provider reports its capabilities. Subversion is read-only, and the case sensitivity of FTP file names comes from the Server tab.
  - A connection or login failure shows as an error on that folder, never as an empty folder (*inferred*).
  - Passwords are stored only when Save password is on. Otherwise the user is asked when connecting (*inferred*).
  - **Profile-kind contract** (owned by E21-S1). Every kind collects secrets through one shared Save-password control, so kinds added later store secrets the same way without extra work.

---

## Stories

### E21-S1 — FTP sites and the Profiles manager
**Priority**: Should · **Layer**: 2 · **Size**: L · **Platforms**: All
**Needs**: E04-S1 (data-source contract), E08-S1 (Folder Compare view)
**Traces**: F-7.4, F-7.5, F-7.6
**Not in this story**: SFTP/FTPS, keys, certificates and TLS → E21-S3; advanced Connection, Proxy, Listing, Transfer, FTP trace and touch local files → E21-S2; Browse Using Profile and Quick Connect → E21-S6; other profile kinds → E21-S4, E21-S5, E21-S7, E21-S8, E21-S9.

As a developer deploying a site over FTP, I can compare my local build folder with the FTP site so that I see which files differ before I upload.

**Acceptance criteria**
- [ ] `ftp://` specs work as base folders in Folder Compare. All of these forms are accepted:
  - `ftp://username@ftp.hostname.com`
  - `ftp://username:password@ftp.hostname.com`
  - `ftp://ftp.hostname.com` (anonymous)
  - `ftp://username@ftp.hostname.com/subfolder` (relative to the login folder, *inferred*)
  - `ftp://username@ftp.hostname.com//subfolder` (absolute, *inferred*)
- [ ] Tools > Profiles lists all profiles, and can create, edit and delete them.
- [ ] The FTP profile Login tab has Port (override), Host, Username, Password, Save password (stored as decided in T7) and Description.
- [ ] The Server tab has Filenames are case sensitive, Encoding for listings and commands, Time zone of the server (timestamps converted to local time), and Custom login commands.
- [ ] The Connection tab has the basics: Read timeout in seconds, and Passive (PASV).
- [ ] A `<default>` FTP profile holds settings that named FTP profiles inherit unless they override them.
- [ ] The Global tab's Anonymous-login email address is used for anonymous logins.
- [ ] `profile:MyProfile` works anywhere a base folder is accepted. `profile:MyProfile?MyFolder/MyFile.ext` resolves to that file through the data source (contract-level test).
- [ ] An FTP site behaves as a data source. It browses and compares in Folder Compare, and supports read, write, delete, rename and create folder through the data-source contract (proven by contract tests).
- [ ] A connection or login failure shows as a folder error carrying the server's message. When no password is saved, the user is asked for one on connect (*inferred*).

- [ ] On macOS, *Profiles* is in the application menu. On Windows and Linux, it is *Tools > Profiles*.
- [ ] Every password field uses the shared Save-password control.
#### Tasks
- **E21-S1-T1** — FTP data source: login, LIST parsing, read, write, delete, rename and create folder · size: L · depends: — · flags: `licence-check`, `craft-libs`
  - AC: contract tests against a test FTP server pass for every operation. Listings map names, sizes and modified times correctly.
- **E21-S1-T2** — `ftp://` URI forms, including relative and absolute subfolders · size: S · depends: E21-S1-T1
  - AC: each specified form opens the expected folder on a test server.
- **E21-S1-T3** — Profiles dialog: list, create, edit and delete · size: M · depends: — · flags: `naming-check`
  - AC: a created FTP profile survives a restart. Deleting it removes it from the list and from `profile:` resolution.
- **E21-S1-T4** — FTP profile tabs (Login, Server, Connection basics, Global) and `<default>` inheritance · size: M · depends: E21-S1-T3
  - AC: changing Passive in `<default>` changes a named profile that doesn't override it. The server time zone shifts displayed timestamps by the configured offset.
- **E21-S1-T5** — `profile:` specs for folders and files · size: S · depends: E21-S1-T1, E21-S1-T3
  - AC: `profile:MySite` compares against a local folder, and `profile:MySite?docs/a.txt` resolves to that file.
- **E21-S1-T6** — Password prompting and connection errors · size: S · depends: E21-S1-T1
  - AC: with Save password off, connecting asks for the password. A wrong password shows a folder error rather than an empty folder.
- **E21-S1-T7** — Decide where saved passwords and keys are stored (PO addition; not in the catalogue) · size: S · depends: — · flags: `decision`
  - AC: decision recorded in this story. Proposed default: the OS credential store (Windows Credential Manager, macOS Keychain, Linux Secret Service) where available, falling back to locally encrypted storage with a visible warning that it is less secure.
- **E21-S1-T8** — Shared Save-password control · size: S · depends: E21-S1-T3 · flags: `licence-check`, `craft-libs`
  - AC: an FTP profile and a test harness kind both save their secret through the shared control into the store decided in T7. With Save password off, neither stores anything.

### E21-S2 — Advanced FTP settings
**Priority**: Could · **Layer**: 4 · **Size**: M · **Platforms**: All
**Needs**: E21-S1 (FTP sites and profiles), E09-S11 (operation log panel), E21-S3 (SFTP and FTPS, for OpenSSH upload tuning)
**Traces**: F-7.4, F-26.7, F-26.11, F-8.6

As an IT administrator syncing to an old or locked-down FTP server, I can tune connections, proxies, listings and transfers so that comparisons and uploads work with that server.

**Acceptance criteria**
- [ ] The Connection tab adds Simultaneous connections (1–10), a limited port range for active (PORT) mode, and Use HOST before login.
- [ ] The Proxy tab has Use proxy, Proxy type (firewall type), Host, Port, Username and Password.
- [ ] The Listing tab has:
  - Link resolution: Fast (a name with an extension is a file, anything else a folder) or Simple (try to change into each link).
  - Use MLSD, which gives high-resolution timestamps. When MLSD works, the LIST options below are ignored, links are listed as plain entries, and listing isn't recursive.
  - LIST options: Show hidden `-a`, Force long format `-l`, Complete timestamps `-T`, Resolve links `-L` and Recursive `-R`.
  - Hide VMS versions.
  - Fetch incomplete timestamps, which gets the full modified time when a listing gives only a date. Not every server supports it.
- [ ] The Transfer tab has:
  - Transfer type ASCII, Binary or Auto. Auto picks the mode by extension, using the Global tab's ASCII types list. ASCII mode handles line endings and EBCDIC→ANSI.
  - Copy timestamps on upload, and Copy Unix permissions on upload.
  - Compress transfers (MODE Z).
  - Limit download/upload, in Kbps per connection.
  - Aggressive uploads to OpenSSH servers, which uses larger buffers and pipelining on SSH connections (*inferred* scope).
- [ ] The folder Handling option Touch local files when copying to an FTP site sets the local file's timestamp to match the server copy, for servers that can't set timestamps.
- [ ] The log panel gains an FTP trace category that logs the protocol traffic.
- [ ] The Use IPv6 when available tweak makes connections use IPv6 where the server offers it.

#### Tasks
- **E21-S2-T1** — Connection and Proxy tabs · size: M · depends: —
  - AC: with Simultaneous connections 3, downloading 30 files uses at most 3 connections. A connection through a configured test proxy succeeds.
- **E21-S2-T2** — Listing tab options, including the MLSD fallback and link resolution · size: M · depends: —
  - AC: when MLSD is on and supported, timestamps carry seconds. On a server without MLSD, the LIST options apply. Fast link resolution treats `lib` as a folder and `a.so` as a file.
- **E21-S2-T3** — Transfer tab and the ASCII types list · size: M · depends: —
  - AC: in Auto mode, a `.txt` on the ASCII types list uploads with converted line endings, while a `.png` uploads byte-identical.
- **E21-S2-T4** — Touch local files when copying to an FTP site · size: S · depends: —
  - AC: after an upload to a server that can't set timestamps, the local file's modified time equals the server copy's, and the pair compares as same.
- **E21-S2-T5** — FTP trace log category and the Use IPv6 tweak · size: S · depends: —
  - AC: with FTP trace on, a connection logs its commands and replies with passwords masked (*inferred*). With IPv6 on, an IPv6-only test server connects.

### E21-S3 — SFTP and FTPS with key and certificate authentication
**Priority**: Should · **Layer**: 3 · **Size**: L · **Platforms**: All
**Needs**: E21-S1 (FTP sites and profiles)
**Traces**: F-7.4, §3.1

As a developer whose servers accept only secure protocols, I can connect over SFTP or FTPS with passwords, SSH keys or client certificates so that I can compare against production safely.

**Acceptance criteria**
- [ ] Host keys and server certificates are verified as settled in T1.
- [ ] `sftp://username@hostname.com` and `ftps://username@hostname.com` specs work just like `ftp://`.
- [ ] The Login tab's Protocol offers FTP, SSL and SSH.
- [ ] The Global tab's SSH public key authentication accepts a private key file in OpenSSH, SSH.COM, PuTTY or PGP format. Create Keys generates a key pair, and keys held in ssh-agent or pageant are used automatically.
- [ ] The Global tab's SSL authentication accepts a client certificate file. Create Certificate makes a self-signed certificate.
- [ ] The Connection tab adds, for SSL only, Clear control channel and Clear data channel. It also adds Limit TLS/SSL versions with a minimum and maximum; by default the connection falls back automatically.
- [ ] Custom login commands don't apply to SFTP.

- [ ] This profile kind's saved secrets go through the shared Save-password control (E21-S1's profile-kind contract).
#### Tasks
- **E21-S3-T1** — Decide how host keys and server certificates are verified (not covered in the catalogue) · size: S · depends: — · flags: `decision`
  - AC: decision recorded. Proposed default:
    - SFTP shows the host-key fingerprint on first connect and remembers accepted keys. It refuses to connect, with a warning, when a known key changes.
    - FTPS validates the server certificate against the OS trust store, and asks before trusting an untrusted certificate.
- **E21-S3-T2** — SFTP data source with password and key authentication · size: L · depends: E21-S3-T1 · flags: `licence-check`, `craft-libs`
  - AC: contract tests pass against a test SSH server three ways: with a password, with an OpenSSH key file, and with a key loaded in ssh-agent.
- **E21-S3-T3** — FTPS data source (explicit TLS, *inferred*) with an optional client certificate · size: M · depends: E21-S3-T1 · flags: `licence-check`, `craft-libs`
  - AC: contract tests pass against a test FTPS server. With Clear data channel on, data transfers after login run unencrypted.
- **E21-S3-T4** — Key formats and Create Keys · size: M · depends: E21-S3-T2 · flags: `licence-check`
  - AC: OpenSSH, SSH.COM, PuTTY and PGP key files each authenticate, and Create Keys writes a usable key pair.
- **E21-S3-T5** — Create Certificate and Limit TLS/SSL versions · size: S · depends: E21-S3-T3 · flags: `licence-check`
  - AC: a self-signed certificate made by Create Certificate authenticates to a server that trusts it. With the minimum set to TLS 1.2, a server that offers only TLS 1.0 is refused.

### E21-S4 — Amazon S3 profiles
**Priority**: Should · **Layer**: 3 · **Size**: M · **Platforms**: All
**Needs**: E21-S1 (FTP sites and profiles)
**Traces**: F-7.5

As a release engineer publishing build artefacts to S3, I can compare a local folder with an S3 bucket so that I see what an upload would change.

**Acceptance criteria**
- [ ] An Amazon S3 profile offers three authentication types:
  - **Saved S3 credentials**: an Access key ID and a Secret access key.
  - **Load from credentials file**: the AWS SDK or CLI credentials or config file. The default is `~/.aws/credentials` or `%UserProfile%\.aws\credentials`. `AWS_SHARED_CREDENTIALS_FILE` and `AWS_CONFIG_FILE` are respected, the profile name defaults to `default`, and session tokens are supported.
  - **Load from environment variables**: `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY` and `AWS_SESSION_TOKEN`, falling back to the credentials file. `AWS_PROFILE` selects a profile.
- [ ] An optional Bucket is available for users who can't list all buckets. The profile also has a Description.
- [ ] `profile:MyS3` opens as a folder. Buckets show as top-level folders, or the configured bucket's contents show directly, and key prefixes show as folders (*inferred*).
- [ ] S3 objects can be read, written and deleted through the data-source contract.
- [ ] *Saved S3 credentials* go through the shared Save-password control, so they are stored the same way as saved passwords (*inferred*).

#### Tasks
- **E21-S4-T1** — S3 data source: buckets and prefixes as folders, with read, write and delete · size: L · depends: — · flags: `licence-check`, `craft-libs`
  - AC: contract tests pass against an S3-compatible test endpoint.
- **E21-S4-T2** — Saved credentials and the Bucket setting · size: S · depends: E21-S4-T1
  - AC: with a Bucket set and no permission to list all buckets, the profile opens that bucket.
- **E21-S4-T3** — Credentials-file and environment-variable authentication · size: M · depends: E21-S4-T1
  - AC: `AWS_PROFILE=dev` selects the `dev` profile from the credentials file, and a session token from the environment is used.

### E21-S5 — WebDAV profiles
**Priority**: Should · **Layer**: 3 · **Size**: M · **Platforms**: All
**Needs**: E21-S1 (FTP sites and profiles)
**Traces**: F-7.5

As a developer whose team shares files on a WebDAV server, I can compare a WebDAV folder with a local one so that I can keep the two in step.

**Acceptance criteria**
- [ ] A WebDAV profile has URL, Username, Password, Recursive listings and Description.
- [ ] `profile:MyDav` opens the URL as a folder. Read, write, delete, rename and create folder work through the data-source contract.
- [ ] With Recursive listings on, the whole tree is fetched in one listing request (*inferred* mechanism). With it off, folders are listed one at a time.
- [ ] An authentication failure shows as a folder error. Save password works as it does for FTP profiles (*inferred*).

- [ ] This profile kind's saved secrets go through the shared Save-password control (E21-S1's profile-kind contract).
#### Tasks
- **E21-S5-T1** — WebDAV data source with every contract operation · size: M · depends: — · flags: `licence-check`, `craft-libs`
  - AC: contract tests pass against a test WebDAV server.
- **E21-S5-T2** — WebDAV profile settings, including Recursive listings · size: S · depends: E21-S5-T1
  - AC: with Recursive listings on, expanding a deep tree makes one listing request (*inferred*). With it off, it makes one request per folder.

### E21-S6 — Browse using profiles and Quick Connect
**Priority**: Should · **Layer**: 3 · **Size**: M · **Platforms**: All
**Needs**: E21-S1 (FTP sites and profiles), E07-S1 (Text Compare view), E15-S5 (table saving), E16-S2 (hex saving)
**Traces**: F-7.6
**Not in this story**: Quick Connect for Subversion → E21-S9.

As a developer, I can browse remote locations from any folder input and connect ad hoc without saving a profile so that remote comparisons are as easy as local ones.

**Acceptance criteria**
- [ ] The Browse for Folder dropdown has Browse Using Profile. It lists Local File System, Quick Connect and every named profile.
- [ ] Quick Connect takes FTP login details. Connect & Browse then opens a folder browser on the server, and the chosen folder fills the path edit. Nothing is saved as a named profile.
- [ ] Choosing a named profile browses that profile's folders.
- [ ] Text Compare has two hidden-by-default commands: File > Open File Using Profile, and Save File Using Profile As. They open and save a file through a profile's folder browser.

- [ ] Table Compare and Hex Compare offer the same hidden-by-default *Open File Using Profile* and *Save File Using Profile As*.
#### Tasks
- **E21-S6-T1** — Browse Using Profile in the Browse for Folder dropdown · size: M · depends: — · flags: `naming-check`
  - AC: picking a profile shows its folder tree. Picking a folder fills the path edit with a `profile:` spec.
- **E21-S6-T2** — Quick Connect for ad-hoc FTP · size: S · depends: E21-S6-T1 · flags: `naming-check`
  - AC: Connect & Browse with valid details browses the server. After it's closed, the Profiles dialog shows no new profile.
- **E21-S6-T3** — Open File Using Profile and Save File Using Profile As in Text Compare · size: M · depends: E21-S6-T1
  - AC: a file opened through a profile is compared. Save File Using Profile As writes the file to the chosen remote folder.
- **E21-S6-T4** — Profile-based open and save in Table and Hex views · size: S · depends: E21-S6-T1
  - AC: a CSV opened through an FTP profile can be edited and saved back through the same profile. A binary file opened the same way in Hex Compare can be too.

### E21-S7 — Dropbox profiles
**Priority**: Could · **Layer**: 3 · **Size**: M · **Platforms**: All
**Needs**: E21-S1 (FTP sites and profiles)
**Traces**: F-7.5

As a designer keeping project files in Dropbox, I can compare a Dropbox folder with a local copy so that I see what changed on either side.

**Acceptance criteria**
- [ ] A Dropbox profile is authorised by logging in through the browser and pasting the code it shows back into the app. The profile also has a Description.
- [ ] `profile:MyDropbox` opens the Dropbox root as a folder. Read, write, delete, rename and create folder work through the data-source contract.
- [ ] An expired or revoked authorisation shows an error that offers a way to authorise again (*inferred*).

- [ ] This profile kind's saved secrets go through the shared Save-password control (E21-S1's profile-kind contract).
#### Tasks
- **E21-S7-T1** — Browser authorisation with a pasted code, with the token stored in the profile · size: M · depends: — · flags: `licence-check`
  - AC: after a valid code is pasted, the profile connects without asking again, even after a restart.
- **E21-S7-T2** — Dropbox data source · size: M · depends: E21-S7-T1 · flags: `craft-libs`
  - AC: contract tests pass against a Dropbox test account or mock.

### E21-S8 — OneDrive profiles
**Priority**: Could · **Layer**: 3 · **Size**: M · **Platforms**: All
**Needs**: E21-S1 (FTP sites and profiles)
**Traces**: F-7.5

As a team lead whose documents live in OneDrive, I can compare a OneDrive folder with a local folder so that I can check that both copies match.

**Acceptance criteria**
- [ ] A OneDrive profile is authorised by logging in through the browser and clicking to allow access. The profile also has a Description.
- [ ] `profile:MyOneDrive` opens the OneDrive root as a folder. Read, write, delete, rename and create folder work through the data-source contract.
- [ ] An expired or revoked authorisation shows an error that offers a way to authorise again (*inferred*).

- [ ] This profile kind's saved secrets go through the shared Save-password control (E21-S1's profile-kind contract).
#### Tasks
- **E21-S8-T1** — Browser authorisation that completes when access is allowed · size: M · depends: — · flags: `licence-check`
  - AC: after access is allowed in the browser, the profile is authorised with no code to copy.
- **E21-S8-T2** — OneDrive data source · size: M · depends: E21-S8-T1 · flags: `craft-libs`
  - AC: contract tests pass against a OneDrive test account or mock.

### E21-S9 — Subversion repositories
**Priority**: Could · **Layer**: 4 · **Size**: M · **Platforms**: All
**Needs**: E21-S1 (FTP sites and profiles), E21-S6 (Browse Using Profile and Quick Connect)
**Traces**: F-7.5, F-7.6

As a developer, I can compare a working folder with any revision in a Subversion repository so that I see what changed since that revision without checking it out.

**Acceptance criteria**
- [ ] A Subversion profile has Repository URL, Revision and Description. The repository is read-only.
- [ ] These specs open repositories directly: `svn://host/path`, `svn://host/path@1234` (revision 1234), `svn+http://`, `svn+https://` and `svn+ssh://`.
- [ ] With no revision given, the latest revision is used (*inferred*).
- [ ] Because the source is read-only, operations that would write to it are refused, and file views open its files read-only.
- [ ] Quick Connect also offers Subversion: enter the repository details, then Connect & Browse.
#### Tasks
- **E21-S9-T1** — Read-only Subversion data source for `svn`, `svn+http`, `svn+https` and `svn+ssh`, with revisions · size: L · depends: — · flags: `licence-check`, `craft-libs`
  - AC: listing and reading at `@1234` return that revision's tree and contents. Write operations report that the source is read-only.
- **E21-S9-T2** — Subversion profile and direct `svn` specs · size: S · depends: E21-S9-T1
  - AC: a profile with Revision 1234 shows the same tree as the spec `svn://host/path@1234`.
- **E21-S9-T3** — Quick Connect for Subversion · size: S · depends: E21-S9-T1
  - AC: Connect & Browse with a repository URL browses the repository without saving a profile.
