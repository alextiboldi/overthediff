# Asset attribution

Every non-code asset in this repository (images, icons, fonts, fixtures, sample files) is listed here with its author, source and licence, in the same change that adds it (AGENTS.md rule 24). Third-party licence notices go in `NOTICE`.

| Asset | Author | Source | Licence | Notes |
|---|---|---|---|---|
| Ubuntu Light (`Ubuntu-Light.ttf`) | Canonical Ltd.; designed by Dalton Maag Ltd | egui's `epaint_default_fonts` crate, storytold/egui at the workspace's pinned revision | Ubuntu-font-1.0 | Embedded in the app through eframe's `default_fonts`; unmodified. |
| Hack Regular (`Hack-Regular.ttf`) | Source Foundry Authors; Bitstream Inc. (Vera); DejaVu project | egui's `epaint_default_fonts` crate, as above | MIT AND Bitstream-Vera | Embedded as above; unmodified. |
| egui-icons (`egui-icons.ttf`) | John Slegers (emoji-icon-font); subset by the egui authors | egui's `epaint_default_fonts` crate, as above | MIT | Five glyphs egui shows as `egui::special_emojis`; embedded as above. |
| Lucide `x` and `plus` icons | Lucide Icons and Contributors; derived from Feather by Cole Bemis | https://github.com/lucide-icons/lucide `icons/x.svg` and `icons/plus.svg` at commit 9fcde889d8ed0c6f739cff76e086c27e072e83ec | ISC (Lucide), MIT (Feather) | Their path coordinates are constants in `crates/ui/src/lib.rs`, drawn as line segments for the tab strip's close and New Tab buttons; unmodified geometry. |

The fonts come in with a dependency rather than as files in this repository, and so does the CLDR number and plural-rule data (Unicode, Inc., Unicode-3.0) that craft-i18n and the ICU4X crates embed for the translation lookup; their notice is in `NOTICE`. Translation catalogs are not in this repository: storytold/craft-i18n holds them, and a build embeds them only when `CRAFT_I18N_DIR` is set. Code dependencies, such as the recycle-bin crate `trash`, rustix and windows-sys for the local data source, craft-keymap, craft-menu and craft-quit (with muda on macOS) for the commands, menus and shortcuts, craft-dialogs and rfd for the path edit's OS file and folder pickers, and craft-settings, toml_writer, dirs, dirs-sys, redox_users and option-ext (MPL-2.0, used unmodified) for the settings store, are not assets: their notices, and the fonts', are in `NOTICE`. The disk images the local data source's tests build are made by the tests at run time and not committed.
