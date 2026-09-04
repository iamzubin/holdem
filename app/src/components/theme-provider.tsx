import { createContext, useCallback, useContext, useEffect, useState } from "react"
import { emit, listen } from "@tauri-apps/api/event"

export type Theme = "dark" | "light" | "system"

type ThemeProviderProps = {
  children: React.ReactNode
  defaultTheme?: Theme
  storageKey?: string
}

type ThemeProviderState = {
  theme: Theme
  setTheme: (theme: Theme) => void
}

const initialState: ThemeProviderState = {
  theme: "system",
  setTheme: () => null,
}

const ThemeProviderContext = createContext<ThemeProviderState>(initialState)

/** Cross-window broadcast channel. `emit` reaches every Tauri WebviewWindow. */
export const THEME_CHANGED_EVENT = "theme-changed"

function isTheme(value: unknown): value is Theme {
  return value === "dark" || value === "light" || value === "system"
}

function resolveTheme(theme: Theme): "dark" | "light" {
  if (theme !== "system") return theme
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light"
}

function applyThemeToDom(theme: Theme) {
  const root = window.document.documentElement
  const resolved = resolveTheme(theme)
  root.classList.remove("light", "dark")
  root.classList.add(resolved)
  root.style.colorScheme = resolved
}

function readInitialTheme(storageKey: string, fallback: Theme): Theme {
  // Newly created windows (popup/settings/…) may carry the opener's theme
  // as `?theme=` so they paint correctly even before any event arrives.
  try {
    const param = new URLSearchParams(window.location.search).get("theme")
    if (isTheme(param)) {
      // First paint only — do not persist here. The opener's value can lag an
      // in-flight `theme-changed` broadcast; persisting it would clobber the
      // fresher store. The broadcast listener below owns persistence.
      return param
    }
  } catch {
    // URL parsing unavailable — fall through to storage
  }
  try {
    const stored = localStorage.getItem(storageKey)
    if (isTheme(stored)) return stored
  } catch {
    // storage unavailable — fall through to fallback
  }
  return fallback
}

function persistTheme(storageKey: string, theme: Theme) {
  try {
    localStorage.setItem(storageKey, theme)
  } catch {
    // non-fatal: theme still applies to this window via state/DOM
  }
}

export function ThemeProvider({
  children,
  defaultTheme = "system",
  storageKey = "vite-ui-theme",
  ...props
}: ThemeProviderProps) {
  const [theme, setThemeState] = useState<Theme>(() => readInitialTheme(storageKey, defaultTheme))

  // Paint (and repaint) this window whenever the theme value changes.
  useEffect(() => {
    applyThemeToDom(theme)
  }, [theme])

  // Follow the OS when the user picked "system".
  useEffect(() => {
    if (theme !== "system") return
    const media = window.matchMedia("(prefers-color-scheme: dark)")
    const onChange = () => applyThemeToDom("system")
    media.addEventListener("change", onChange)
    return () => media.removeEventListener("change", onChange)
  }, [theme])

  // Cross-window sync:
  // - `storage` covers plain browser tabs sharing one localStorage.
  // - the Tauri event covers Tauri WebviewWindows (main/popup/settings/…),
  //   each of which runs its own React tree with its own provider state.
  // Receivers apply without re-emitting, so there is no event loop.
  useEffect(() => {
    const onStorage = (event: StorageEvent) => {
      if (event.key !== storageKey || !isTheme(event.newValue)) return
      setThemeState((current) => (current === event.newValue ? current : (event.newValue as Theme)))
    }
    window.addEventListener("storage", onStorage)

    let unlistenTheme: (() => void) | undefined
    let cancelled = false
    listen<Theme>(THEME_CHANGED_EVENT, (event) => {
      if (!isTheme(event.payload)) return
      persistTheme(storageKey, event.payload)
      setThemeState((current) => (current === event.payload ? current : event.payload))
    })
      .then((unlisten) => {
        if (cancelled) unlisten()
        else unlistenTheme = unlisten
        // A broadcast fired before `listen` attached would otherwise be missed
        // (leaving a possibly stale `?theme=` seed). Re-read the shared store
        // once subscribed; a harmless no-op when storages are isolated.
        try {
          const stored = localStorage.getItem(storageKey)
          if (isTheme(stored)) setThemeState((current) => (current === stored ? current : stored))
        } catch {
          // storage unavailable — keep param-derived state
        }
      })
      .catch(() => {
        // Not running inside Tauri (plain browser dev) — `storage` listener above still syncs tabs.
      })

    return () => {
      cancelled = true
      window.removeEventListener("storage", onStorage)
      unlistenTheme?.()
    }
  }, [storageKey])

  const setTheme = useCallback(
    (next: Theme) => {
      persistTheme(storageKey, next)
      setThemeState(next)
      // Broadcast to every other window (main ↔ settings ↔ popup …).
      // Our own listener will also see this emit and no-op on the equal value.
      emit(THEME_CHANGED_EVENT, next).catch(() => {
        // Plain browser / emit unavailable — local `storage` event still syncs other tabs.
      })
    },
    [storageKey],
  )

  const value = {
    theme,
    setTheme,
  }

  return (
    <ThemeProviderContext.Provider {...props} value={value}>
      {children}
    </ThemeProviderContext.Provider>
  )
}

export const useTheme = () => {
  const context = useContext(ThemeProviderContext)

  if (context === undefined)
    throw new Error("useTheme must be used within a ThemeProvider")

  return context
}
