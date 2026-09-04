import type { Metadata } from 'next'
import Link from 'next/link'
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
      <Link href="/" className="text-sm text-zinc-500 hover:text-zinc-900 dark:hover:text-zinc-100">
        ← Back to Holdem
      </Link>
      <p className="mt-6 text-sm font-medium uppercase tracking-widest text-zinc-500">Download</p>
      <h1 className="mb-3 text-3xl font-bold md:text-4xl">Download Holdem for Windows — free</h1>
      <p className="text-zinc-600 dark:text-zinc-400">
        Holdem is 100% free and open-source (MIT). No account, no trial countdown,
        no wait timers. Grab the installer below or pick a version from{' '}
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

      <div className="mt-8 rounded-2xl border border-zinc-200 p-6 text-center dark:border-zinc-800">
        <ButtonLink variant="primary" size="lg" href={DOWNLOAD_URL_FALLBACK} download>
          ⬇️ Download Holdem for Windows (x64)
        </ButtonLink>
        <p className="mt-3 text-xs text-zinc-500">
          Current release: v3.0.0 · Windows 10/11 x64 · .exe installer ·{' '}
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

      <h2 className="mb-4 mt-10 text-2xl font-bold">System requirements</h2>
      <ul className="list-disc space-y-2 pl-6 text-zinc-600 dark:text-zinc-400">
        <li>Windows 10 or Windows 11, 64-bit</li>
        <li>Runs quietly in the system tray; optional auto-launch on startup</li>
      </ul>

      <h2 className="mb-4 mt-10 text-2xl font-bold">Install in 3 steps</h2>
      <ol className="list-decimal space-y-2 pl-6 text-zinc-600 dark:text-zinc-400">
        <li>Download the .exe installer above.</li>
        <li>Run it and follow the setup prompts.</li>
        <li>
          Drag a file, shake your mouse, and the shelf appears — see the{' '}
          <Link href="/docs" className="underline">docs</Link> for the full walkthrough.
        </li>
      </ol>

      <hr className="my-10" />
      <p className="text-sm text-zinc-500">
        <Link href="/pricing">Pricing — free forever</Link> •{' '}
        <Link href="/changelog">Changelog</Link> • <Link href="/">Homepage →</Link>
      </p>
    </main>
  )
}
