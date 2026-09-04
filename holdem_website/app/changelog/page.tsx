import type { Metadata } from 'next'
import Link from 'next/link'
import { GITHUB_REPO } from '@/lib/constants'

export const metadata: Metadata = {
  title: 'Changelog — Holdem Release Notes',
  description:
    'Holdem changelog and release notes. Current release v3.1.1. Full version history on GitHub Releases.',
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
      <Link href="/" className="text-ink-subtle hover:text-ink text-sm">
        ← Back to Holdem
      </Link>
      <p className="text-ink-subtle mt-6 text-sm font-medium tracking-widest uppercase">
        Changelog
      </p>
      <h1 className="mb-3 text-3xl font-bold md:text-4xl">Release notes</h1>
      <p className="text-ink-muted">
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
        <div className="border-hairline bg-surface-1 rounded-xl border p-5">
          <div className="flex items-baseline justify-between gap-3">
            <h2 className="text-xl font-bold">v3.1.1</h2>
            <span className="bg-success/15 text-success shrink-0 rounded-full px-3 py-1 text-xs font-medium">
              Latest
            </span>
          </div>
          <p className="text-ink-subtle mt-1 text-sm">
            See GitHub Releases for the full notes.
          </p>
          <p className="text-ink-muted mt-2">
            Bug-fix release for file drops: dropping large folders no longer
            freezes the shelf, files and images dropped from browsers and
            other apps land reliably, and fixed blank thumbnails for some
            images.
          </p>
          <a
            href={`${GITHUB_REPO}/releases`}
            target="_blank"
            rel="noopener noreferrer"
            className="mt-3 inline-block text-sm underline"
          >
            View v3.1.1 on GitHub →
          </a>
        </div>

        <div className="border-hairline bg-surface-1 rounded-xl border p-5">
          <div className="flex items-baseline justify-between gap-3">
            <h2 className="text-xl font-bold">v3.1.0</h2>
          </div>
          <p className="text-ink-subtle mt-1 text-sm">
            See GitHub Releases for the full notes.
          </p>
          <p className="text-ink-muted mt-2">
            Added comprehensive internationalization (i18n) support across 25
            languages with automatic browser locale detection and RTL layout.
            Upgraded native Windows OLE drag-and-drop target to support URL
            drops, text snippets, and browser-dragged images directly onto the
            shelf, with improved unlistener lifecycle handling.
          </p>
          <a
            href={`${GITHUB_REPO}/releases`}
            target="_blank"
            rel="noopener noreferrer"
            className="mt-3 inline-block text-sm underline"
          >
            View v3.1.0 on GitHub →
          </a>
        </div>

        <div className="border-hairline bg-surface-1 rounded-xl border p-5">
          <div className="flex items-baseline justify-between gap-3">
            <h2 className="text-xl font-bold">v3.0.0</h2>
          </div>
          <p className="text-ink-subtle mt-1 text-sm">
            See GitHub Releases for the full notes.
          </p>
          <p className="text-ink-muted mt-2">
            Major Windows release featuring the floating file shelf, mouse-shake
            summon gesture, global hotkey, browser-image drops, system-tray
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

      <h2 className="mt-10 mb-4 text-2xl font-bold">On the roadmap</h2>
      <ul className="text-ink-muted list-disc space-y-2 pl-6">
        <li>Multiple shelves for different tasks</li>
        <li>Custom theme and opacity controls</li>
      </ul>

      <hr className="my-10" />
      <p className="text-ink-subtle text-sm">
        <Link href="/download">Download</Link> •{' '}
        <Link href="/pricing">Pricing</Link> • <Link href="/">Homepage →</Link>
      </p>
    </main>
  )
}
