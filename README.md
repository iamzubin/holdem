<p align="center">
  <h1 align="center">Holdem</h1>
  <p align="center">
    <strong>The free Dropover for Windows.</strong><br />
    A lightweight floating shelf to stash files mid-drag — shake, drop, done.
  </p>
</p>

<p align="center">
  <a href="https://github.com/iamzubin/holdem/releases"><img src="https://img.shields.io/github/downloads/iamzubin/holdem/total.svg?style=flat-square" alt="Downloads" /></a>
  <a href="https://github.com/iamzubin/holdem/releases/latest"><img src="https://img.shields.io/github/v/release/iamzubin/holdem?style=flat-square" alt="Latest release" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/github/license/iamzubin/holdem?style=flat-square" alt="License: MIT" /></a>
  <a href="https://github.com/iamzubin/holdem/releases"><img src="https://img.shields.io/badge/platform-Windows_10_|_11-blue?style=flat-square" alt="Platform: Windows" /></a>
  <a href="https://github.com/iamzubin/holdem/stargazers"><img src="https://img.shields.io/github/stars/iamzubin/holdem?style=flat-square" alt="GitHub stars" /></a>
</p>

<p align="center">
  <a href="https://github.com/iamzubin/holdem/releases/download/3.1.0/holdem_3.1.0_x64-setup.exe"><strong>⬇ Download for Windows — Free</strong></a><br />
  <sub>Windows 10/11 x64 · v3.1.0 · Free & open-source (MIT) · <a href="https://holdem.iamzub.in">Website</a> · <a href="https://github.com/iamzubin/holdem/releases">All releases</a></sub>
</p>

<p align="center">
  <img src="./assets/screen.gif" alt="Holdem demo — shake to summon the shelf, stash files, drop anywhere" width="720" />
</p>

---

## Why Holdem?

Moving files on Windows means juggling Explorer windows, losing your drag mid-way, or dropping things on the Desktop “for now”.

Holdem fixes that with a temporary floating shelf:

1. **Shake** your mouse while dragging — the shelf appears at your cursor
2. **Drop** files, folders, or browser images onto it, let go of the mouse
3. **Navigate** anywhere, then drag everything out where you need it

Built with [Tauri](https://tauri.app) + Rust for low memory use. Runs quietly in your system tray. No paywall, no wait timers — unlike Dropshelf free tier or macOS-only Dropover.

> 🌐 Learn more: [holdem.iamzub.in](https://holdem.iamzub.in) · [Docs](https://holdem.iamzub.in/docs) · [Changelog](https://holdem.iamzub.in/changelog) · [Pricing — free forever](https://holdem.iamzub.in/pricing)

## ✨ Features

|  | What you get |
|---|---|
| 🗂️ | **Floating holding area** — draggable, always-on-top shelf for files & folders |
| 🖱️ | **Shake to summon** — start dragging, shake, shelf appears at cursor |
| ⌨️ | **Global hotkey** — summon the shelf anytime (customizable) |
| 🖼️ | **Browser image drops** — drag images straight from Chrome / Edge / Firefox |
| 🔔 | **Tray integration** — right-click for settings, quick actions |
| 🚀 | **Auto-launch + auto-update** — ready on login, always up to date |
| 🌍 | **25 languages** — auto-detects locale, RTL layout support |

**What you can stash:**
- Files & folders from File Explorer
- Images dragged from the browser

**On the roadmap:**
- URL / text-snippet drops
- Multiple shelves for different tasks
- Custom theme & opacity controls

See [Changelog](https://holdem.iamzub.in/changelog) for what's new in v3.1.0.

## 📥 Download

**Recommended:** grab the signed installer from GitHub Releases:

👉 **[Download holdem_3.1.0_x64-setup.exe](https://github.com/iamzubin/holdem/releases/download/3.1.0/holdem_3.1.0_x64-setup.exe)**

Or browse [all releases](https://github.com/iamzubin/holdem/releases) for notes and older versions.

## 🚀 Quick start

1. Install & launch Holdem (it sits in your system tray)
2. Start dragging files in Explorer
3. Shake your mouse — drop files onto the shelf
4. Switch folders / apps / desktops freely
5. Drag items out of the shelf to finish — it auto-hides when empty

> Tip: right-click the tray icon for settings, hotkey, and auto-launch. Collect files from several folders, then drop them all at once.

Full guide: [holdem.iamzub.in/docs](https://holdem.iamzub.in/docs)

## 🛠️ Run from source

Prerequisites: [Rust](https://www.rust-lang.org/tools/install) + [Node.js](https://nodejs.org/)

```bash
git clone https://github.com/iamzubin/holdem.git
cd holdem/app
npm install
npm run tauri dev
```

Build a local installer:

```bash
npm run tauri build
```

Rust checks (in `app/src-tauri`):

```bash
cargo test
cargo clippy -- -D warnings
```

Project layout:

```
├── app/              # Tauri desktop app (React + TS + Rust)
│   ├── src/          # React frontend
│   └── src-tauri/    # Rust backend
├── holdem_website/   # Marketing site (Next.js) — holdem.iamzub.in
└── assets/           # Demo gif + icons for README
```

## ❓ FAQ

**Is Holdem really free?**
Yes — 100% free and MIT open-source. No pro unlocks, no wait timers.

**Windows versions?**
Windows 10 and 11 (x64).

**Dropover / Dropshelf / Yoink alternative?**
Yes — those are Mac-only or freemium. Holdem brings the same shelf workflow to Windows, free.
Comparisons: [vs Dropover](https://holdem.iamzub.in/vs/dropover) · [vs Dropshelf](https://holdem.iamzub.in/vs/dropshelf) · [vs Yoink](https://holdem.iamzub.in/vs/yoink) · [vs DropPoint](https://holdem.iamzub.in/vs/droppoint)

**How do I summon it?**
Shake while dragging, or press the global hotkey.

## 🤝 Contributing

We welcome bug reports, feature requests, and PRs!

- Open an [issue](https://github.com/iamzubin/holdem/issues) or submit a PR
- Translations via [Crowdin](https://crowdin.com/project/holdem) (`crowdin.yml`)
- Please run `cargo clippy -- -D warnings` and `npm run build` before submitting

If Holdem saves you time, ⭐ star the repo and share it with a friend.

## 📄 License

MIT — see [LICENSE](LICENSE).

Made with Tauri, Rust, and ❤️ by [Zubin](https://github.com/iamzubin) · [Website](https://holdem.iamzub.in) · [Releases](https://github.com/iamzubin/holdem/releases)
