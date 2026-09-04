import type { Metadata } from 'next'
import Link from 'next/link'
import { GITHUB_REPO } from '@/lib/constants'

export const metadata: Metadata = {
  title: 'Changelog — Holdem Release Notes',
  description:
    'Holdem changelog and release notes. Current release v3.0.0. Full version history on GitHub Releases.',
  alternates: {
    canonical: 'https://holdem.iamzub.in/changelog',
  },
  openGraph: {
    title: 'Changelog — Holdem Release Notes',
    description: 'Holdem release notes. Full history on GitHub Releases.',
    url: 'https://holdem.iamzub.in/changelog',
    images: [
      {
        url: '/og.png',
        width: 1280,
        height: 640,
        alt: 'Holdem changelog',
      },
    ],
    type: 'website',
  },
}

export default function ChangelogPage() {
  return (
    <main className="mx-auto max-w-3xl py-8">
      <Link href="/" className="text-sm text-zinc-500 hover:text-zinc-900 dark:hover:text-zinc-100">
        ← Back to Holdem
      </Link>
      <p className="mt-6 text-sm font-medium uppercase tracking-widest text-zinc-500">Changelog</p>
      <h1 className="mb-3 text-3xl font-bold md:text-4xl">Release notes</h1>
      <p className="text-zinc-600 dark:text-zinc-400">
        The source of truth for every Holdem release is{' '}
        <a
          href={`${GITHUB_REPO}/releases`}
          target="_blank"
          rel="noopener noreferrer"
          className="underline"
        >
          GitHub Releases
        </a>
        , where each version ships with full notes and the Windows installer.
      </p>

      <div className="mt-8 space-y-4">
        <div className="rounded-xl border border-zinc-200 p-5 dark:border-zinc-800">
          <div className="flex items-baseline justify-between gap-3">
            <h2 className="text-xl font-bold">v3.0.0</h2>
            <span className="shrink-0 rounded-full bg-green-100 px-3 py-1 text-xs font-medium text-green-800 dark:bg-green-900 dark:text-green-100">
              Latest
            </span>
          </div>
          <p className="mt-1 text-sm text-zinc-500">See GitHub Releases for the full notes.</p>
          <p className="mt-2 text-zinc-600 dark:text-zinc-400">
            Latest stable Windows release with the floating file shelf,
            mouse-shake summon, global hotkey, browser-image drops, system-tray
            integration, and auto-launch on startup.
          </p>
          <a
            href={`${GITHUB_REPO}/releases`}
            target="_blank"
            rel="noopener noreferrer"
            className="mt-3 inline-block text-sm underline"
          >
            View v3.0.0 on GitHub →
          </a>
        </div>
      </div>

      <h2 className="mb-4 mt-10 text-2xl font-bold">On the roadmap</h2>
      <ul className="list-disc space-y-2 pl-6 text-zinc-600 dark:text-zinc-400">
        <li>URL and text-snippet drops (files, folders, and web images work today)</li>
        <li>Multiple shelves for different tasks</li>
      </ul>

      <hr className="my-10" />
      <p className="text-sm text-zinc-500">
        <Link href="/download">Download</Link> • <Link href="/pricing">Pricing</Link> •{' '}
        <Link href="/">Homepage →</Link>
      </p>
    </main>
  )
}
