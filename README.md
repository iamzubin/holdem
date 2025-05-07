# Holdem

Holdem is an open-source tool inspired by Dropover for macOS, built using [Tauri](https://tauri.app). It simplifies file drag-and-drop management, offering a temporary holding area for files or folders, making multitasking and file organization smoother across your desktop.

![Holdem](./assets/screen.gif)


## Features

- **Temporary Holding Area:** Hold and manage multiple files or folders in a convenient floating window.
- **Drag and Drop Support:** Drag files or folders into the Holdem window, and drag them out when needed.
- **Customizable Window Positioning:** Easily move and adjust the Holdem window anywhere on the screen.

## Download

<center><a href="https://github.com/iamzubin/holdem/releases"><img src="./assets/icons8-download-48.png"></a></center>
<center>Download the latest release from the <a href="https://github.com/iamzubin/holdem/releases">releases page</a></center>


## Installation

### From Source

Make sure you have cargo (rust) and pnpm (nodejs) installed.

1. Clone the repository:
   ```bash
   git clone git@github.com:iamzubin/holdem.git
   ```
2. Navigate to the project directory:
   ```bash
   cd holdem/app
   ```
3. Install dependencies:
   ```bash
   pnpm install
   ```
4. Build and run the application:
   ```bash
   pnpm tauri dev
   ```

## Usage

1. Launch Holdem.
2. Shake your mouse like crazy while holding some files
3. Drag files into the floating window that appears.
4. Files will remain in the holding area until dragged out.


## Features

- [x] Basic drag and drop.
- [x] Tray icon.
- [x] Global hotkey.
- [x] Auto start on startup.
- [ ] Drop data(images, text, etc) to the holding area.
- [ ] Multiple Shelfs.

## Contributing

We welcome contributions! If you want to report a bug, request a feature, or submit a pull request, please open an issue or submit a PR.

## License

Holdem is licensed under the [MIT License](LICENSE).

---
