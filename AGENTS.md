# AGENTS.md

This repository contains three main projects:
1. **app/** - Tauri desktop app (React + TypeScript + Vite + Tailwind)
2. **holdem_website/** - Next.js marketing website (Next.js 15 + React 19 + Tailwind CSS)
3. **thumb-rs/** - Rust library for cross-platform thumbnail extraction (see thumb-rs/AGENTS.md)

---

## Development Commands

### app/ (Tauri Desktop App)

```bash
cd app

# Install dependencies
npm install

# user will do npm run dev, you don't do it.

# Build
npm run build         # TypeScript check + Vite build
npm run tauri build  # Production build

# Tauri specific
npm run tauri        # Run tauri commands
```

### holdem_website/ (Next.js Website)

```bash
cd holdem_website

# Install dependencies
npm install

# Development
npm run dev           # Start Next.js dev server (port 3000)

# Build & Test
npm run build         # Production build
npm run start         # Start production server
npm run lint          # Run ESLint

# Format
npx prettier --write .
```

### thumb-rs/ (Rust Library)

```bash
cd thumb-rs

# Build & Test
cargo build           # Compile
cargo test            # Run tests
cargo check           # Type check
cargo clippy -- -D warnings  # Lint with warnings as errors
```

### app/src-tauri/ (Rust Backend)

```bash
cd app/src-tauri

cargo build
cargo test
cargo clippy -- -D warnings
```

---

## Code Style Guidelines

### General Conventions

- **File Naming**: kebab-case for files (e.g., `file-utils.ts`, `use-file-management.ts`)
- **Component Naming**: PascalCase for React components (e.g., `App.tsx`, `FileIcon.tsx`)
- **Function Naming**: camelCase for functions and variables
- **Import Order**:
  1. External libraries (React, Tauri, Radix UI)
  2. Internal components
  3. Hooks
  4. Lib/utilities
  5. Types

### TypeScript

- Use explicit types for function parameters and return types when not obvious
- Prefer `interface` for object shapes, `type` for unions/intersections
- Use `strict: true` where possible (currently disabled in `app/tsconfig.json`)
- Use path aliases: `@/*` maps to `./src/*`

### React

- Use functional components with hooks
- Prefer `useCallback` and `useMemo` for performance optimization
- Use `clsx` and `tailwind-merge` for conditional className handling
- Use Radix UI primitives (/@radix-ui/*) for accessible components

### Tailwind CSS

- Use utility classes for styling
- Use `@apply` sparingly in CSS files
- Follow mobile-first responsive design
- Use `tailwind-merge` with `clsx` for conditional classes:

```typescript
import { clsx } from "clsx";
import { twMerge } from "tailwind-merge";

function cn(...inputs: (string | undefined | null | false)[]) {
  return twMerge(clsx(inputs));
}
```

### Rust (thumb-rs & src-tauri)

- Use explicit types in function signatures
- Use `thiserror` for error handling with `Result<T, Error>`
- Use `Send + Sync` bounds where necessary
- Prefer `std::path::Path` or `AsRef<Path>` for input arguments
- Run `cargo clippy -- -D warnings` before committing

### Imports (app/)

Use path aliases with `@/` prefix:

```typescript
// Components
import { Button } from "@/components/ui/button";
import { DynamicFileIcon } from "@/components/FileIcon";

// Hooks
import { useFileManagement } from "@/hooks/useFileManagement";

// Lib
import { handleMultiFileDragStart } from "@/lib/fileUtils";
import { closeWindow } from "@/lib/windowUtils";

// Types
import { FilePreview, FileWithPath } from "@/types";
```

### Formatting (holdem_website/)

Prettier configuration (`.prettierrc.json`):
- Trailing commas: all
- Semicolons: no
- Tab width: 2
- Single quotes: yes
- Print width: 80
- Arrow parens: always

Run formatting:
```bash
npx prettier --write .
```

### Error Handling

- Use try-catch for async operations with proper error logging
- Console errors should include context: `console.error('Failed to check config existence:', error)`
- Use Tauri invoke with `.catch()` for command errors
- In Rust, use `thiserror` derive for error types

### Key Dependencies

**app/**:
- Tauri 2.x with plugins (fs, dialog, shell, updater, etc.)
- React 18 + React Router DOM
- Radix UI primitives
- Tailwind CSS 3.x
- dnd-kit for drag-and-drop

**holdem_website/**:
- Next.js 15 (App Router)
- React 19
- Motion (framer-motion)
- Tailwind CSS 4.x
- MDX support

---

## Project Structure

```
/                           # Root
├── app/                    # Tauri desktop app
│   ├── src/               # React frontend
│   │   ├── components/   # UI components
│   │   ├── hooks/        # Custom React hooks
│   │   ├── lib/          # Utilities
│   │   ├── pages/        # Page components
│   │   └── types.ts      # TypeScript types
│   └── src-tauri/        # Rust backend
│       └── src/
│           └── lib.rs    # Tauri commands
├── holdem_website/        # Next.js marketing site
│   ├── app/              # App router pages
│   ├── components/       # React components
│   └── hooks/            # Custom hooks
└── thumb-rs/             # Rust thumbnail library
    ├── src/
    │   └── platform/     # Platform-specific implementations
    └── examples/         # CLI example
```

---

## Common Tasks

### Running a Single Test (Rust)

```bash
# In thumb-rs or app/src-tauri
cargo test <test_name>
# Example: cargo test test_get_thumbnail
```

### Adding a New Tauri Command

1. Add command to `app/src-tauri/src/lib.rs` with `#[tauri::command]`
2. Import and call from frontend using `invoke('command_name')`
3. Add TypeScript type if needed in frontend

### Adding a New UI Component

1. Create component in `app/src/components/`
2. Use Radix UI primitives when available
3. Use `cn()` utility for className merging
4. Follow existing component patterns (e.g., Button, Dialog)

---

## Notes

- The `app/` project uses Tauri 2.x with staticlib, cdylib, and rlib crate types
- The `thumb-rs` library has its own AGENTS.md with specific Rust guidelines
- Both TypeScript projects use Tailwind but different versions (3.x vs 4.x)
- The holdem_website uses the new Tailwind 4 configuration (CSS-based)
