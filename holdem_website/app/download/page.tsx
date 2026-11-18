import type { Metadata } from 'next'
import Link from 'next/link'
import { Download } from 'lucide-react'
import { ButtonLink } from '../components/ui/button'
import { StoreBadge } from '../components/store-badge'
import {
  DOWNLOAD_URL_FALLBACK,
  GITHUB_REPO,
  STORE_PRICE,
  STORE_TRIAL_DAYS,
} from '@/lib/constants'

export const metadata: Metadata = {
  title: 'Download Holdem for Windows',
  description:
    'Get Holdem, the Dropover for Windows: from the Microsoft Store ($4.99, 7-day free trial) or free and open source from GitHub. Drag-and-drop file shelf for Windows 10/11 (x64).',
  alternates: {
    canonical: 'https://holdem.iamzub.in/download',
  },
  openGraph: {
    title: 'Download Holdem for Windows',
    description:
      'Microsoft Store or free from GitHub. Open-source file shelf for Windows 10/11.',
    url: 'https://holdem.iamzub.in/download',
    images: [
      {
        url: '/og.png',
        width: 1280,
        height: 640,
        alt: 'Holdem — drag-and-drop file shelf for Windows',
      },
    ],
    type: 'website',
  },
}

export default function DownloadPage() {
  return (
    <main className="mx-auto max-w-3xl py-8">
      <Link href="/" className="text-ink-subtle hover:text-ink text-sm">
        ← Back to Holdem
      </Link>
      <p className="text-ink-subtle mt-6 text-sm font-medium tracking-widest uppercase">
        Download
      </p>
      <h1 className="mb-3 text-3xl font-bold md:text-4xl">
        Download Holdem for Windows
      </h1>
      <p className="text-ink-muted">
        Holdem is open source (MIT) and needs no account. Get it from the
        Microsoft Store, or download the same app free from{' '}
        <a
          href={`${GITHUB_REPO}/releases`}
          target="_blank"
          rel="noopener noreferrer"
          className="underline"
        >
          GitHub Releases
        </a>
        .
      </p>

      <div className="mt-8 grid gap-4 md:grid-cols-2">
        <div className="border-hairline bg-surface-1 flex flex-col rounded-2xl border p-6">
          <h2 className="text-lg font-bold">Microsoft Store</h2>
          <p className="text-ink-subtle text-sm">
            {STORE_PRICE} · {STORE_TRIAL_DAYS}-day free trial
          </p>
          <ul className="text-ink-muted mt-3 mb-5 flex-1 list-disc space-y-1 pl-5 text-sm">
            <li>Installs and updates through the Store</li>
            <li>Install on every PC signed in to your Microsoft account</li>
            <li>Pays for continued development</li>
          </ul>
          <StoreBadge height={44} />
        </div>
        <div className="border-hairline bg-surface-1 flex flex-col rounded-2xl border p-6">
          <h2 className="text-lg font-bold">GitHub</h2>
          <p className="text-ink-subtle text-sm">Free · MIT open source</p>
          <ul className="text-ink-muted mt-3 mb-5 flex-1 list-disc space-y-1 pl-5 text-sm">
            <li>The same app, with every feature</li>
            <li>.exe installer that updates itself</li>
            <li>No timers, nothing locked</li>
          </ul>
          <ButtonLink
            variant="secondary"
            size="lg"
            href={DOWNLOAD_URL_FALLBACK}
            download
            className="self-start"
          >
            <Download className="h-5 w-5" /> Download .exe (x64)
          </ButtonLink>
        </div>
      </div>
      <p className="text-ink-subtle mt-3 text-xs">
        Current release: v3.2.0 · Windows 10/11 x64 ·{' '}
        <a
          href={`${GITHUB_REPO}/releases`}
          target="_blank"
          rel="noopener noreferrer"
          className="underline"
        >
          All releases &amp; notes
        </a>
      </p>

      <h2 className="mt-10 mb-4 text-2xl font-bold">System requirements</h2>
      <ul className="text-ink-muted list-disc space-y-2 pl-6">
        <li>Windows 10 version 1809 or later, or Windows 11, 64-bit</li>
        <li>
          Runs quietly in the system tray; optional auto-launch on startup
        </li>
      </ul>

      <h2 className="mt-10 mb-4 text-2xl font-bold">Install in 3 steps</h2>
      <ol className="text-ink-muted list-decimal space-y-2 pl-6">
        <li>
          Click <strong>Get</strong> on the Microsoft Store page, or download
          the .exe installer above.
        </li>
        <li>If you used the .exe, run it and follow the setup prompts.</li>
        <li>
          Drag a file, shake your mouse, and the shelf appears — see the{' '}
          <Link href="/docs" className="underline">
            docs
          </Link>{' '}
          for the full walkthrough.
        </li>
      </ol>

      <hr className="my-10" />
      <p className="text-ink-subtle text-sm">
        <Link href="/pricing">Pricing</Link> •{' '}
        <Link href="/changelog">Changelog</Link> •{' '}
        <Link href="/">Homepage →</Link>
      </p>
    </main>
  )
}
