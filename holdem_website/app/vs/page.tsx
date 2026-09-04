import type { Metadata } from 'next'
import Link from 'next/link'

export const metadata: Metadata = {
  title: 'Compare Holdem vs Dropover, Yoink, Dropshelf & DropPoint',
  description:
    'See how Holdem — the free, open-source drag-and-drop shelf for Windows — stacks up against Dropover, Yoink, Dropshelf, and DropPoint.',
  alternates: {
    canonical: 'https://holdem.iamzub.in/vs',
  },
}

const COMPARISONS = [
  {
    href: '/vs/dropover',
    title: 'Holdem vs Dropover',
    blurb:
      'Dropover is Mac-only ($6.99 Pro). Holdem is the free Windows equivalent — full comparison.',
  },
  {
    href: '/vs/yoink',
    title: 'Holdem vs Yoink',
    blurb:
      'Yoink is Mac-only ($8.99). Holdem brings the shelf workflow to Windows, free & open-source.',
  },
  {
    href: '/vs/dropshelf',
    title: 'Holdem vs Dropshelf',
    blurb:
      'The best free Dropshelf alternative — no 3-second wait, 100% open source.',
  },
  {
    href: '/vs/droppoint',
    title: 'Holdem vs DropPoint',
    blurb:
      'Tauri/Rust vs Electron — clean, minimal vs dated & clunky. Modern upgrade.',
  },
]

export default function VsIndex() {
  return (
    <main className="mx-auto max-w-3xl py-8">
      <Link
        href="/"
        className="text-sm text-zinc-500 hover:text-zinc-900 dark:hover:text-zinc-100"
      >
        ← Back to Holdem
      </Link>
      <h1 className="mt-6 text-2xl font-bold">Compare Holdem</h1>
      <p className="mt-2 text-zinc-600 dark:text-zinc-400">
        See how Holdem stacks up against popular alternatives.
      </p>
      <div className="mt-6 grid gap-3">
        {COMPARISONS.map((c) => (
          <Link
            key={c.href}
            href={c.href}
            className="rounded-xl border border-zinc-200 p-4 hover:bg-zinc-50 dark:border-zinc-800 dark:hover:bg-zinc-900"
          >
            <div className="font-medium">{c.title}</div>
            <div className="text-sm text-zinc-500">{c.blurb}</div>
          </Link>
        ))}
      </div>
    </main>
  )
}
