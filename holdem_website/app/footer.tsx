'use client'
import { TextLoop } from '@/components/ui/text-loop'
import Link from 'next/link'

const FOOTER_LINKS = [
  { href: '/download', label: 'Download' },
  { href: '/pricing', label: 'Pricing' },
  { href: '/changelog', label: 'Changelog' },
  { href: '/docs', label: 'Docs' },
  { href: '/vs/dropshelf', label: 'Holdem vs Dropshelf' },
  { href: '/vs/droppoint', label: 'Holdem vs DropPoint' },
  { href: '/vs/dropover', label: 'Holdem vs Dropover' },
  { href: '/vs/yoink', label: 'Holdem vs Yoink' },
]

export function Footer() {
  const year = new Date().getFullYear()
  return (
    <footer className="mt-24 border-t border-hairline px-0 py-8">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <a
          href="https://github.com/iamzubin/holdem"
          target="_blank"
          className="inline-flex w-[170px] shrink-0 justify-start"
        >
          <TextLoop className="text-xs text-ink-subtle">
            <span>© {year} Holdem.</span>
            <span>Built with Tauri.</span>
          </TextLoop>
        </a>
        <div className="flex flex-wrap items-center gap-x-4 gap-y-1 text-xs">
          {FOOTER_LINKS.map((l) => (
            <Link
              key={l.href}
              href={l.href}
              className="text-ink-subtle hover:text-ink hover:underline"
            >
              {l.label}
            </Link>
          ))}
        </div>
      </div>
    </footer>
  )
}
