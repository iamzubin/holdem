# Changelog

## \[2.1.0]

- [`229aa3e`](https://github.com/crabnebula-dev/drag-rs/commit/229aa3e26c85c31074abd3b4a4538b2ec65eb094) Added `mode` option to `drag::Options` to either copy or move a file.
- [`1e7adfd`](https://github.com/crabnebula-dev/drag-rs/commit/1e7adfd9662bc1be6d369f438ee749e908dee02a) Fix crash on macOS systems running Intel or older macOS releases.

## \[2.0.0]

- [`244887f`](https://github.com/crabnebula-dev/drag-rs/commit/244887fa36b12ac615919b9d2d149edca3d1f1c7) Update to tauri v2.

## \[0.4.0]

- [`639e0fd`](https://github.com/crabnebula-dev/drag-rs/commit/639e0fd801109d88007d0aeafe04367cdc251eb7) Added the cursor position of the drop event as the `start_drag` callback closure second argument.
- [`639e0fd`](https://github.com/crabnebula-dev/drag-rs/commit/639e0fd801109d88007d0aeafe04367cdc251eb7) Added `Options` as the last argument of the `start_drag` function.

## \[0.3.0]

- [`f58ed78`](https://github.com/crabnebula-dev/drag-rs/commit/f58ed7838abe1fe5b23c4e3aa92df28e77564345) Added `DragItem::Drag` variant (supported on macOS) to drag a buffer (e.g. Final Cut Pro XMLs).
- [`1449076`](https://github.com/crabnebula-dev/drag-rs/commit/14490764de8ff50969a3f2299d204e44e091752e) The `start_drag` function now takes a closure for the operation result (either `DragResult::Dropped` or `DragResult::Cancelled`).

## \[0.2.0]

- [`644cfa2`](https://github.com/crabnebula-dev/drag-rs/commit/644cfa28b09bee9c3de396bdcc1dc801a26d65bc) Initial release.
