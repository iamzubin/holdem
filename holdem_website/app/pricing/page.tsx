import type { Metadata } from 'next'
import Link from 'next/link'
import { DOWNLOAD_URL_FALLBACK } from '@/lib/constants'

export const metadata: Metadata = {
  title: 'Pricing — Holdem Is Free Forever',
  description:
    'Holdem pricing: 100% free & open-source, forever. Compare with Dropover Pro ($6.99), Yoink ($8.99), Dropshelf Pro, and DropPoint.',
  alternates: {
    canonical: 'https://holdem.iamzub.in/pricing',
  },
  openGraph: {
    title: 'Pricing — Holdem Is Free Forever',
    description: 'Holdem is free & open-source forever. See how it compares.',
    url: 'https://holdem.iamzub.in/pricing',
    images: [
      {
        url: '/og.png',
        width: 1280,
        height: 640,
        alt: 'Holdem pricing — free forever',
      },
    ],
    type: 'website',
  },
}

const faqSchema = {
  '@context': 'https://schema.org',
  '@type': 'FAQPage',
  mainEntity: [
    {
      '@type': 'Question',
      name: 'Is Holdem really free?',
      acceptedAnswer: {
        '@type': 'Answer',
        text: 'Yes. Holdem is MIT-licensed open-source software: free to download, free to use, no wait timers, no pro tier.',
      },
    },
    {
      '@type': 'Question',
      name: 'How much do Dropover and Yoink cost?',
      acceptedAnswer: {
        '@type': 'Answer',
        text: 'Per their Mac App Store listings, Dropover Pro is a $6.99 one-time in-app purchase and Yoink is a $8.99 one-time purchase. Both are Mac-only. Prices may change — check the listings.',
      },
    },
  ],
}

export default function PricingPage() {
  return (
    <main className="mx-auto max-w-3xl py-8">
      <script
        type="application/ld+json"
        dangerouslySetInnerHTML={{ __html: JSON.stringify(faqSchema) }}
      />
      <Link href="/" className="text-sm text-zinc-500 hover:text-zinc-900 dark:hover:text-zinc-100">
        ← Back to Holdem
      </Link>
      <p className="mt-6 text-sm font-medium uppercase tracking-widest text-zinc-500">Pricing</p>
      <h1 className="mb-3 text-3xl font-bold md:text-4xl">Holdem is free. Forever.</h1>
      <p className="text-zinc-600 dark:text-zinc-400">
        Holdem is MIT-licensed open source: free to download, free to use, no
        wait timers, no “Pro” unlock. If you are comparing shelves, here is what
        the alternatives cost — checked September 2026, always confirm on the
        official listing before buying.
      </p>

      <div className="mt-8 overflow-x-auto rounded-xl border border-zinc-200 dark:border-zinc-800">
        <table className="w-full text-left text-sm">
          <thead className="bg-zinc-50 text-zinc-500 dark:bg-zinc-900">
            <tr>
              <th className="px-4 py-3 font-medium">App</th>
              <th className="px-4 py-3 font-medium">Price</th>
              <th className="px-4 py-3 font-medium">Platform</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-zinc-200 dark:divide-zinc-800">
            <tr>
              <td className="px-4 py-3 font-medium">Holdem</td>
              <td className="px-4 py-3">Free, open-source (MIT) — no paywall</td>
              <td className="px-4 py-3">Windows 10/11</td>
            </tr>
            <tr>
              <td className="px-4 py-3 font-medium">Dropover</td>
              <td className="px-4 py-3">
                Free download, 14-day full trial, then 3-second wait unless you buy{' '}
                <a
                  href="https://apps.apple.com/us/app/dropover-easier-drag-drop/id1355679052"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="underline"
                >
                  Dropover Pro ($6.99, one-time, Mac App Store)
                </a>
              </td>
              <td className="px-4 py-3">macOS only</td>
            </tr>
            <tr>
              <td className="px-4 py-3 font-medium">Yoink</td>
              <td className="px-4 py-3">
                <a
                  href="https://eternalstorms.at/yoink/mac/"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="underline"
                >
                  $8.99 one-time
                </a>{' '}
                (Mac App Store / direct / Setapp), free trial on the developer site
              </td>
              <td className="px-4 py-3">macOS only</td>
            </tr>
            <tr>
              <td className="px-4 py-3 font-medium">Dropshelf</td>
              <td className="px-4 py-3">
                Free with a 3-second wait;{' '}
                <a
                  href="https://apps.microsoft.com/detail/9mzpc6p14l7n"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="underline"
                >
                  Dropshelf Pro
                </a>{' '}
                is a one-time in-app purchase — check the Microsoft Store listing for
                the current price
              </td>
              <td className="px-4 py-3">Windows</td>
            </tr>
            <tr>
              <td className="px-4 py-3 font-medium">DropPoint</td>
              <td className="px-4 py-3">
                Free, open-source (
                <a
                  href="https://github.com/GameGodS3/DropPoint"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="underline"
                >
                  GPL-3.0
                </a>
                )
              </td>
              <td className="px-4 py-3">Windows / macOS / Linux</td>
            </tr>
          </tbody>
        </table>
      </div>

      <div className="mt-8 rounded-2xl border border-zinc-200 p-6 text-center dark:border-zinc-800">
        <a
          href={DOWNLOAD_URL_FALLBACK}
          download
          className="inline-flex items-center gap-2 rounded-lg bg-zinc-900 px-6 py-3 font-medium text-white shadow hover:bg-zinc-800 dark:bg-white dark:text-zinc-900 dark:hover:bg-zinc-100"
        >
          ⬇️ Download Holdem — $0 forever
        </a>
      </div>

      <hr className="my-10" />
      <p className="text-sm text-zinc-500">
        <Link href="/download">Download</Link> • <Link href="/changelog">Changelog</Link> •{' '}
        <Link href="/">Homepage →</Link>
      </p>
    </main>
  )
}
