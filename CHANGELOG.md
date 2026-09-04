# Changelog

All notable changes to Holdem are documented here. The desktop app follows
SemVer (`3.x.y` tags, no `v` prefix). Website copy that only references the
current version lives in `holdem_website/`; the updater manifest
(`holdem_website/public/update.json`) is synced automatically by CI after each
release — do not hand-edit it.

## [3.2.0] - 2026-09-05

### Added

- Universal native Windows drop target (`IDropTarget`, Tauri `dragDropEnabled`
  off): virtual files (`FileGroupDescriptor`), bitmaps (`CF_DIB`/`PNG`), web
  images, HTML, URLs, and text snippets land in a dated drop folder with unique
  timestamp+rand names.
- Instant folder drops: folders store size 0 instead of recursive size walks,
  so huge trees no longer stall the shelf.
- Single batched `files_updated` emit per `add_files` batch.
- In-house thumbnail engine: thumb-rs Windows `IShellItemImageFactory` backend
  vendored into `app/src-tauri/src/thumbnail.rs` (fixed 256px, `thiserror`
  `ThumbnailError`, regression tests).
- OpenCode automation: auto PR reviews (`opencode-review.yml`, free
  `muse-spark-1.3` model at `xhigh`) and on-demand `/oc` tasks
  (`opencode.yml`), plus a `/resolve-threads` playbook.
- Website Linear-style redesign: dark-only canvas (`#010102`), surface ladder
  + hairline borders, lavender CTA (`#5e6ad2`), Inter type, 1280px container,
  emoji-free Lucide feature icons, touch-aware demo.
- README refresh with hero, features, download CTA, and FAQ.

### Fixed

- OLE/HDROP double-free, `CF_HTML` fragment UTF-8 boundary panics, unaligned
  UTF-16 decoding UB, hostile DIB dimensions (`i32::MIN` height, unchecked
  `w*h*4`), invisible 32bpp BI_RGB PNGs, `GetDIBits` misuse, unbounded image
  downloads (25MB streamed cap), poisoned `FileList` mutex handling.

### Removed

- Unpinned `thumb-rs` git dependency and `thumb-rs` submodule (fully in-house
  now); unused `active-win-pos-rs` dependency and macOS-only transitive deps
  from Windows builds.

## [3.1.0] - 2026-09-04

- Windows OLE native drop target: files, web images, URLs, and text snippets.
- Internationalization (i18n): 25 locales, browser-locale detection, RTL
  layout, unlistener lifecycle cleanup.
- Website: shared button system, SEO overhaul (sitemap, JSON-LD, OG/Twitter,
  analytics), docs/pricing/changelog/`vs/*` comparison routes.

## [3.0.0] - 2026-07-02

- Major Windows release: floating file shelf, mouse-shake summon gesture,
  global hotkey, browser-image drops, system-tray integration, auto-launch on
  startup, macOS support groundwork.
