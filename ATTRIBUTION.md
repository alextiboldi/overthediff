# Asset attribution

Every non-code asset in this repository (images, icons, fonts, fixtures, sample files) is listed here with its author, source and licence, in the same change that adds it (AGENTS.md rule 24). Third-party licence notices go in `NOTICE`.

| Asset | Author | Source | Licence | Notes |
|---|---|---|---|---|
| Ubuntu Light (`Ubuntu-Light.ttf`) | Canonical Ltd.; designed by Dalton Maag Ltd | egui's `epaint_default_fonts` crate, storytold/egui at the workspace's pinned revision | Ubuntu-font-1.0 | Embedded in the app through eframe's `default_fonts`; unmodified. |
| Hack Regular (`Hack-Regular.ttf`) | Source Foundry Authors; Bitstream Inc. (Vera); DejaVu project | egui's `epaint_default_fonts` crate, as above | MIT AND Bitstream-Vera | Embedded as above; unmodified. |
| egui-icons (`egui-icons.ttf`) | John Slegers (emoji-icon-font); subset by the egui authors | egui's `epaint_default_fonts` crate, as above | MIT | Five glyphs egui shows as `egui::special_emojis`; embedded as above. |
| Lucide `x` and `plus` icons | Lucide Icons and Contributors; derived from Feather by Cole Bemis | https://github.com/lucide-icons/lucide `icons/x.svg` and `icons/plus.svg` at commit 9fcde889d8ed0c6f739cff76e086c27e072e83ec | ISC (Lucide), MIT (Feather) | Their path coordinates are constants in `crates/ui/src/lib.rs`, drawn as line segments for the tab strip's close and New Tab buttons; unmodified geometry. |

The fonts come in with a dependency rather than as files in this repository. Code dependencies, such as the recycle-bin crate `trash`, and rustix and windows-sys for the local data source, are not assets: their notices, and the fonts', are in `NOTICE`. The disk images the local data source's tests build are made by the tests at run time and not committed.
