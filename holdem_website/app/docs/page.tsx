import type { Metadata } from 'next'
import Link from 'next/link'
import { DOWNLOAD_URL_FALLBACK } from '@/lib/constants'

export const metadata: Metadata = {
  title: 'Docs — How to Use Holdem on Windows',
  description:
    'Holdem docs: install the free file shelf for Windows, summon it with a mouse shake or hotkey, stash files, and drop them anywhere. FAQ included.',
  alternates: {
    canonical: 'https://holdem.iamzub.in/docs',
  },
  openGraph: {
    title: 'Docs — How to Use Holdem on Windows',
    description: 'Install, summon, stash, drop. The Holdem quick-start guide.',
    url: 'https://holdem.iamzub.in/docs',
    images: [
      {
        url: '/og.png',
        width: 1280,
        height: 640,
        alt: 'Holdem docs — how to use the file shelf',
      },
    ],
    type: 'website',
  },
}

export default function DocsPage() {
  return (
    <main className="mx-auto max-w-3xl py-8">
      <Link href="/" className="text-sm text-zinc-500 hover:text-zinc-900 dark:hover:text-zinc-100">
        ← Back to Holdem
      </Link>
      <p className="mt-6 text-sm font-medium uppercase tracking-widest text-zinc-500">Docs</p>
      <h1 className="mb-3 text-3xl font-bold md:text-4xl">How to use Holdem</h1>
      <p className="text-zinc-600 dark:text-zinc-400">
        Holdem is a floating shelf for Windows: stash files mid-drag, let go of the
        mouse, navigate anywhere, then drag them out. Here is the whole workflow.
      </p>

      <h2 className="mb-4 mt-10 text-2xl font-bold">1. Install</h2>
      <p className="text-zinc-600 dark:text-zinc-400">
        <a href={DOWNLOAD_URL_FALLBACK} download className="underline">
          Download Holdem for Windows
        </a>{' '}
        (Windows 10/11, 64-bit), run the .exe installer, and Holdem starts running
        quietly in your system tray. Enable auto-launch so it is ready when you log in.
      </p>

      <h2 className="mb-4 mt-10 text-2xl font-bold">2. Summon the shelf</h2>
      <p className="text-zinc-600 dark:text-zinc-400">
        Start dragging files, then <strong>shake your mouse</strong> — the floating
        shelf appears right at your cursor. Prefer keys? Use the{' '}
        <strong>global hotkey</strong> (customizable) to summon it any time.
      </p>

      <h2 className="mb-4 mt-10 text-2xl font-bold">3. Stash, navigate, drop</h2>
      <ol className="list-decimal space-y-2 pl-6 text-zinc-600 dark:text-zinc-400">
        <li>Drop the files you are dragging onto the shelf.</li>
        <li>
          Let go of the mouse. Switch folders, apps, desktops, or virtual desktops freely.
        </li>
        <li>Drag the items out of the shelf to their destination — it auto-hides when empty.</li>
      </ol>

      <h2 className="mb-4 mt-10 text-2xl font-bold">What you can stash</h2>
      <ul className="list-disc space-y-2 pl-6 text-zinc-600 dark:text-zinc-400">
        <li>Files and folders from File Explorer</li>
        <li>Images dragged directly from your web browser</li>
        <li>URL and text-snippet drops are on the roadmap — see the <Link href="/changelog" className="underline">changelog</Link></li>
      </ul>

      <h2 className="mb-4 mt-10 text-2xl font-bold">Tips</h2>
      <ul className="list-disc space-y-2 pl-6 text-zinc-600 dark:text-zinc-400">
        <li>Right-click the tray icon for settings and quick actions.</li>
        <li>Use the shelf to collect files from several folders, then drop them all at once.</li>
        <li>
          Comparing with another shelf? See <Link href="/vs/dropshelf" className="underline">Holdem vs Dropshelf</Link>,{' '}
          <Link href="/vs/dropover" className="underline">Holdem vs Dropover</Link>,{' '}
          <Link href="/vs/yoink" className="underline">Holdem vs Yoink</Link>, and{' '}
          <Link href="/vs/droppoint" className="underline">Holdem vs DropPoint</Link>.
        </li>
      </ul>

      <hr className="my-10" />
      <p className="text-sm text-zinc-500">
        <Link href="/download">Download</Link> • <Link href="/pricing">Pricing</Link> •{' '}
        <Link href="/">Homepage →</Link>
      </p>
    </main>
  )
}
