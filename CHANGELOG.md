# Changelog

All notable changes to Holdem are documented here. The desktop app follows
SemVer (`3.x.y` tags, no `v` prefix). Website copy that only references the
current version lives in `holdem_website/`; the updater manifest
(`holdem_website/public/update.json`) is synced automatically by CI after each
release — do not hand-edit it.

## [3.1.1] - 2026-09-05

### Fixed

- Fixed file drops crashing or freezing the shelf: dropping large folders is
  now instant, and files, images, links, and text dropped from browsers and
  other apps land reliably every time.
- Fixed blank thumbnails showing for some images.

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
