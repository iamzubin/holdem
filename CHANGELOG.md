# Changelog

All notable changes to Holdem are documented here. The desktop app follows
SemVer (`3.x.y` tags, no `v` prefix). Website copy that only references the
current version lives in `holdem_website/`; the updater manifest
(`holdem_website/public/update.json`) is synced manually after each
release — do not hand-edit it.

## [Unreleased]

### Added

- A "What's new" window on the first start after updating from 3.2.0 or
  earlier: what changed, where to find it, and links to support Holdem,
  request a feature or report a problem. New installs don't see it.

### Removed

- Opening the shelf by holding a drag against a screen edge, added in
  3.2.0. Shake while dragging, or press the hotkey, instead.

## [3.2.0] - 2026-11-16

### Added

- Multiple shelves. Add, rename and switch shelves from the tabs in the item
  list or with Ctrl+Tab, and move items between them. Closing a shelf keeps
  it for 7 days: reopen it from a tab's right-click menu or with
  Ctrl+Shift+T.
- The shelf remembers its items across restarts and updates. Items on a
  drive or network share that is offline stay on the shelf, greyed out, and
  work again once it is back. Items whose file was moved or deleted leave the
  shelf with a message saying so.
- Preview any item with Space, and step through items with the arrow keys.
- New actions on items: copy them as files (Ctrl+C) to paste into Explorer,
  mail or chat; copy their paths; copy them to a folder; compress them to a
  ZIP file; pin them so Clear leaves them on the shelf; and Undo after
  removing or clearing.
- Rename items Holdem saved itself (attachments, screenshots, text notes,
  links) with F2.
- Resize and convert images: 50%, 25%, fit within 1920 or 1280 pixels, and
  convert to PNG or JPEG. The result is added as a new item and the original
  is never changed.
- Paste onto the shelf with Ctrl+V, or "Add clipboard to shelf" in the tray
  menu: files, email attachments, images, links and text, as if dropped.
- Optional: open the shelf by holding a drag against a screen edge. Off by
  default; turn it on in Settings.
- Clear messages when a drop can't be added, saying why: the item is too
  large, the app didn't hand over the file, the disk is full, security
  software blocked the save, it's already on the shelf, or Windows blocked a
  drag from an app running as administrator. Click Details for the whole
  explanation.
- Translucent windows: a blur of what's behind Holdem, on Windows 11.
- A welcome window on first start, and a choice of light, dark or system
  theme.

### Changed

- Dragging items out starts when the pointer moves past Windows' drag
  distance, as in Explorer, instead of after a fixed delay. A long click no
  longer starts a drag.
- The item list opens instantly after the first time, and closes when you
  switch to another app instead of on a 3-second timer. It stays open while
  you drop files onto it or pick a folder.
- Settings save as you change them.
- Shaking works in every app except drawing, 3D and CAD tools, where it is
  off by default. Change the list in Settings.
- Dropped links are saved as shortcuts without visiting the site; only links
  straight to an image are downloaded. Saved downloads and attachments are
  marked as coming from the internet, like browser downloads.

### Fixed

- Large email attachments, chat images and files dropped from ZIP folders
  are added reliably, including empty files and files with non-English names.
- Emails dragged out of classic Outlook are added as .msg files.
- Files dragged out of 7-Zip or WinRAR stay on the shelf: Holdem keeps its
  own copy, since those apps delete their temporary files after the drag.
- Saved items you remove (screenshots, notes, attachments) are kept for a
  week before they go to the Recycle Bin, and Holdem asks before deleting
  anything that can't be recycled.
- Black areas in the drag-out image no longer turn see-through.
- Images downloaded from links keep readable names ("My Photo.jpg" instead
  of "My%20Photo.jpg").
- The item list stays on screen near the edges and on monitors with
  different scaling, and its drop target works every time it opens.
- Dragging an item back onto Holdem's own windows no longer counts as a
  drop.
- Many smaller fixes to the hotkey, the tray menu, settings that failed to
  load, keyboard use and screen-reader announcements.

## [3.1.1] - 2026-09-05

### Fixed

- Fixed file drops crashing or freezing the shelf: dropping large folders is
  now instant, and files, images, links, and text dropped from browsers and
  other apps land reliably every time.
- Fixed blank thumbnails showing for some images.
- Fixed popup shelf clipping the last row: the file list now scrolls
  correctly without cutting off the final item.

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
