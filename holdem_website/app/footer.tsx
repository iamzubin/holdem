'use client'
import { AnimatedBackground } from '@/components/ui/animated-background'
import { TextLoop } from '@/components/ui/text-loop'
import { MonitorIcon, MoonIcon, SunIcon } from 'lucide-react'
import { useTheme } from 'next-themes'
import { useEffect, useState } from 'react'
import Link from 'next/link'

const THEMES_OPTIONS = [
  {
    label: 'Light',
    id: 'light',
    icon: <SunIcon className="h-4 w-4" />,
  },
  {
    label: 'Dark',
    id: 'dark',
    icon: <MoonIcon className="h-4 w-4" />,
  },
  {
    label: 'System',
    id: 'system',
    icon: <MonitorIcon className="h-4 w-4" />,
  },
]

function ThemeSwitch() {
  const [mounted, setMounted] = useState(false)
  const { theme, setTheme } = useTheme()

  useEffect(() => {
    setMounted(true)
  }, [])

  if (!mounted) {
    return null
  }

  return (
    <AnimatedBackground
      className="pointer-events-none rounded-lg bg-zinc-100 dark:bg-zinc-800"
      defaultValue={theme}
      transition={{
        type: 'spring',
        bounce: 0,
        duration: 0.2,
      }}
      enableHover={false}
      onValueChange={(id) => {
        setTheme(id as string)
      }}
    >
      {THEMES_OPTIONS.map((theme) => {
        return (
          <button
            key={theme.id}
            className="inline-flex h-7 w-7 items-center justify-center text-zinc-500 transition-colors duration-100 focus-visible:outline-2 data-[checked=true]:text-zinc-950 dark:text-zinc-400 dark:data-[checked=true]:text-zinc-50"
            type="button"
            aria-label={`Switch to ${theme.label} theme`}
            data-id={theme.id}
          >
            {theme.icon}
          </button>
        )
      })}
    </AnimatedBackground>
  )
}

export function Footer() {
  const year = new Date().getFullYear()
  return (
    <footer className="mt-24 border-t border-zinc-100 px-0 py-4 dark:border-zinc-800">
      <div className="grid grid-cols-1 items-center gap-3 sm:grid-cols-3">
        <a
          href="https://github.com/iamzubin/holdem"
          target="_blank"
          className="inline-flex w-[170px] shrink-0 justify-start"
        >
          <TextLoop className="text-xs text-zinc-500">
            <span>© {year} Holdem.</span>
            <span>Built with Tauri and ❤️.</span>
          </TextLoop>
        </a>
        <div className="flex flex-wrap items-center justify-center gap-x-4 gap-y-1 text-xs sm:justify-center">
          <Link
            href="/download"
            className="text-zinc-500 hover:text-zinc-900 hover:underline dark:text-zinc-400 dark:hover:text-zinc-100"
          >
            Download
          </Link>
          <Link
            href="/pricing"
            className="text-zinc-500 hover:text-zinc-900 hover:underline dark:text-zinc-400 dark:hover:text-zinc-100"
          >
            Pricing
          </Link>
          <Link
            href="/changelog"
            className="text-zinc-500 hover:text-zinc-900 hover:underline dark:text-zinc-400 dark:hover:text-zinc-100"
          >
            Changelog
          </Link>
          <Link
            href="/docs"
            className="text-zinc-500 hover:text-zinc-900 hover:underline dark:text-zinc-400 dark:hover:text-zinc-100"
          >
            Docs
          </Link>
          <Link
            href="/vs/dropshelf"
            className="text-zinc-500 hover:text-zinc-900 hover:underline dark:text-zinc-400 dark:hover:text-zinc-100"
          >
            Holdem vs Dropshelf
          </Link>
          <Link
            href="/vs/droppoint"
            className="text-zinc-500 hover:text-zinc-900 hover:underline dark:text-zinc-400 dark:hover:text-zinc-100"
          >
            Holdem vs DropPoint
          </Link>
          <Link
            href="/vs/dropover"
            className="text-zinc-500 hover:text-zinc-900 hover:underline dark:text-zinc-400 dark:hover:text-zinc-100"
          >
            Holdem vs Dropover
          </Link>
          <Link
            href="/vs/yoink"
            className="text-zinc-500 hover:text-zinc-900 hover:underline dark:text-zinc-400 dark:hover:text-zinc-100"
          >
            Holdem vs Yoink
          </Link>
        </div>
        <div className="flex justify-start text-xs text-zinc-400 sm:justify-end">
          <ThemeSwitch />
        </div>
      </div>
    </footer>
  )
}
