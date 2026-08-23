import type { Metadata } from 'next'
import Link from 'next/link'

export const metadata: Metadata = {
  title: 'Holdem vs Dropover: Free Dropover Alternative for Windows',
  description: 'Holdem vs Dropover — free, open-source shelf for Windows. Full comparison coming soon.',
  openGraph: {
    title: 'Holdem vs Dropover: Free Dropover Alternative for Windows',
    description: 'Free, open-source shelf for Windows — Holdem vs Dropover.',
    url: 'https://holdem.iamzub.in/vs/dropover',
    images: [{ url: '/og.png', width: 1200, height: 600, alt: 'Holdem vs Dropover' }],
    type: 'article',
  },
  twitter: {
    card: 'summary_large_image',
    site: '@iamzub_in',
    creator: '@iamzub_in',
    title: 'Holdem vs Dropover: Free Dropover Alternative for Windows',
    description: 'Free, open-source shelf for Windows.',
    images: ['/og.png'],
  },
}

export default function VsDropoverPage() {
  return (
    <main className="mx-auto max-w-3xl py-8">
      <Link href="/" className="text-sm text-zinc-500 hover:text-zinc-900 dark:hover:text-zinc-100">
        ← Back to Holdem
      </Link>
      <article className="prose prose-zinc mt-6 max-w-none dark:prose-invert prose-headings:tracking-tight prose-a:text-zinc-900 dark:prose-a:text-zinc-100">
        <p className="text-sm font-medium uppercase tracking-widest text-zinc-500">Compare • Holdem vs Dropover</p>
        <h1 className="!mb-3 text-3xl font-bold md:text-4xl">Holdem vs Dropover: Free Alternative for Windows</h1>
        <p className="text-zinc-600 dark:text-zinc-400">
          Dropover is a great file shelf for Mac, but there&apos;s no native Windows version. Holdem brings the same shake-to-summon shelf to
          Windows — mouse-shake, hotkey, web images, system tray — with no paywall. Detailed Dropover comparison coming soon. For now, see our{' '}
          <Link href="/vs/dropshelf">Holdem vs Dropshelf</Link> breakdown (same 3-second-free advantage).
        </p>
        <div className="not-prose mt-6 flex gap-3">
          <Link href="/vs/dropshelf" className="rounded-lg bg-zinc-900 px-5 py-2.5 text-sm font-medium text-white dark:bg-white dark:text-zinc-900">
            Read Dropshelf comparison
          </Link>
          <Link href="/" className="rounded-lg border px-5 py-2.5 text-sm font-medium dark:border-zinc-700">
            Go to homepage
          </Link>
        </div>
      </article>
    </main>
  )
}
