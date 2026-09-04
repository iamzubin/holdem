import type { Metadata } from 'next'
import Link from 'next/link'
import { Download } from 'lucide-react'
import { ButtonLink } from '../components/ui/button'
import { DOWNLOAD_URL_FALLBACK, GITHUB_REPO } from '@/lib/constants'

export const metadata: Metadata = {
  title: 'Download Holdem for Windows — Free',
  description:
    'Download Holdem, the free Dropover for Windows. Free & open-source drag-and-drop file shelf for Windows 10/11 (x64). No paywall, no wait timers.',
  alternates: {
    canonical: 'https://holdem.iamzub.in/download',
  },
  openGraph: {
    title: 'Download Holdem for Windows — Free',
    description: 'Free, open-source file shelf for Windows 10/11. No paywall.',
    url: 'https://holdem.iamzub.in/download',
    images: [
      {
        url: '/og.png',
        width: 1280,
        height: 640,
        alt: 'Holdem — free drag-and-drop file shelf for Windows',
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
        Download Holdem for Windows — free
      </h1>
      <p className="text-ink-muted">
        Holdem is 100% free and open-source (MIT). No account, no trial
        countdown, no wait timers. Grab the installer below or pick a version
        from{' '}
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

      <div className="border-hairline bg-surface-1 mt-8 rounded-2xl border p-6 text-center">
        <ButtonLink
          variant="primary"
          size="lg"
          href={DOWNLOAD_URL_FALLBACK}
          download
        >
          <Download className="h-5 w-5" /> Download Holdem for Windows (x64)
        </ButtonLink>
        <p className="text-ink-subtle mt-3 text-xs">
          Current release: v3.2.0 · Windows 10/11 x64 · .exe installer ·{' '}
          <a
            href={`${GITHUB_REPO}/releases`}
            target="_blank"
            rel="noopener noreferrer"
            className="underline"
          >
            All releases &amp; notes
          </a>
        </p>
      </div>

      <h2 className="mt-10 mb-4 text-2xl font-bold">System requirements</h2>
      <ul className="text-ink-muted list-disc space-y-2 pl-6">
        <li>Windows 10 or Windows 11, 64-bit</li>
        <li>
          Runs quietly in the system tray; optional auto-launch on startup
        </li>
      </ul>

      <h2 className="mt-10 mb-4 text-2xl font-bold">Install in 3 steps</h2>
      <ol className="text-ink-muted list-decimal space-y-2 pl-6">
        <li>Download the .exe installer above.</li>
        <li>Run it and follow the setup prompts.</li>
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
        <Link href="/pricing">Pricing — free forever</Link> •{' '}
        <Link href="/changelog">Changelog</Link> •{' '}
        <Link href="/">Homepage →</Link>
      </p>
    </main>
  )
}
