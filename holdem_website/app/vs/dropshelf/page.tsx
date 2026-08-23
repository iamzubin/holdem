import type { Metadata } from 'next'
import Link from 'next/link'

export const metadata: Metadata = {
  title: 'Holdem vs Dropshelf: The Best Free Dropshelf Alternative for Windows',
  description:
    'Holdem vs Dropshelf — 100% free, open-source drag-and-drop shelf for Windows with no 3-second wait timers. See full comparison.',
  openGraph: {
    title: 'Holdem vs Dropshelf: The Best Free Dropshelf Alternative for Windows',
    description: '100% free, open-source shelf for Windows — no 3-second wait. See Holdem vs Dropshelf.',
    url: 'https://holdem.iamzub.in/vs/dropshelf',
    images: [{ url: '/og.png', width: 1280, height: 640, alt: 'Holdem vs Dropshelf' }],
    type: 'article',
  },
  twitter: {
    card: 'summary_large_image',
    site: '@iamzub_in',
    creator: '@iamzub_in',
    title: 'Holdem vs Dropshelf: The Best Free Dropshelf Alternative for Windows',
    description: '100% free, open-source shelf for Windows — no 3-second wait.',
    images: ['/og.png'],
  },
}

export default function VsDropshelfPage() {
  return (
    <main className="mx-auto max-w-3xl py-8">
      <Link href="/" className="text-sm text-zinc-500 hover:text-zinc-900 dark:hover:text-zinc-100">
        ← Back to Holdem
      </Link>

      <article className="prose prose-zinc mt-6 max-w-none dark:prose-invert prose-headings:tracking-tight prose-a:text-zinc-900 dark:prose-a:text-zinc-100">
        <p className="text-sm font-medium uppercase tracking-widest text-zinc-500">Compare • Holdem vs Dropshelf</p>
        <h1 className="!mb-3 text-3xl font-bold md:text-4xl">Holdem vs Dropshelf: The Best Free Dropshelf Alternative for Windows</h1>
        <p className="lead text-zinc-600 dark:text-zinc-400">
          If you work with multiple monitors or frequently move files across crowded desktops, you know the frustration of Windows drag-and-drop.
        </p>

        <p>
          You click a file, drag it to the edge of your screen, clumsily <code>Alt + Tab</code> trying to find your target window, and pray you
          don&apos;t drop it in the wrong place.
        </p>
        <p>
          Mac users have long relied on tools like <strong>Dropover</strong> to solve this by creating a temporary &quot;shelf&quot; for files. If
          you&apos;re searching for a <strong>Dropshelf alternative for Windows</strong>, you&apos;ve likely seen Dropshelf. While Dropshelf is a good
          utility, its free tier hits you with a forced 3-second wait time every time you drop a file. When you&apos;re in a flow state, artificial
          delays kill your productivity.
        </p>
        <p>
          That is why I built <Link href="/">Holdem</Link>.
        </p>

        <h2>What is Holdem?</h2>
        <p>
          Holdem is a lightning-fast, 100% free, and open-source <strong>drag and drop file utility for Windows</strong>. It acts as a temporary
          digital basket for your files. Shake your mouse or hit a global hotkey, drop your files on the shelf, and freely navigate to your target
          destination without holding down the mouse button.
        </p>
        <p>It gets out of your way and lets you work faster.</p>

        <h2>Holdem vs. Dropshelf: Which Should You Use?</h2>
        <p>Here is a quick look at how Holdem compares to Dropshelf for your daily workflow:</p>

        <div className="not-prose overflow-x-auto rounded-xl border border-zinc-200 dark:border-zinc-800">
          <table className="w-full text-left text-sm">
            <thead className="bg-zinc-50 text-zinc-500 dark:bg-zinc-900">
              <tr>
                <th className="px-4 py-3 font-medium">Feature</th>
                <th className="px-4 py-3 font-medium">Holdem</th>
                <th className="px-4 py-3 font-medium">Dropshelf</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-zinc-200 dark:divide-zinc-800">
              <tr>
                <td className="px-4 py-3 font-medium">Pricing</td>
                <td className="px-4 py-3">100% Free &amp; Open Source</td>
                <td className="px-4 py-3">Freemium (Pro upgrade required to remove wait times)</td>
              </tr>
              <tr>
                <td className="px-4 py-3 font-medium">Wait Timers</td>
                <td className="px-4 py-3">None. Completely frictionless.</td>
                <td className="px-4 py-3">3-second forced wait on the free tier</td>
              </tr>
              <tr>
                <td className="px-4 py-3 font-medium">Trigger Mechanism</td>
                <td className="px-4 py-3">Mouse shake or Global Hotkey</td>
                <td className="px-4 py-3">Mouse shake or Keyboard shortcut</td>
              </tr>
              <tr>
                <td className="px-4 py-3 font-medium">Supported Content</td>
                <td className="px-4 py-3">Files, folders, and web images (URLs coming soon!)</td>
                <td className="px-4 py-3">Files, folders, web images, URLs, and text</td>
              </tr>
              <tr>
                <td className="px-4 py-3 font-medium">Philosophy</td>
                <td className="px-4 py-3">Speed, open-source community, and zero paywalls</td>
                <td className="px-4 py-3">Native Windows 11 design, paywalled features</td>
              </tr>
            </tbody>
          </table>
        </div>

        <h2>Designed for Pure Productivity</h2>
        <p>Holdem was built purely for UX and speed. There are no menus to dig through and no premium upgrades to unlock basic functionality.</p>
        <ul>
          <li>
            <strong>Instant Activation:</strong> A quick mouse shake or hotkey summons the shelf exactly where your cursor is.
          </li>
          <li>
            <strong>Web Images Supported:</strong> You can already drag and drop images directly from your browser onto the shelf. Support for URLs
            and text snippets is currently in development and coming very soon.
          </li>
          <li>
            <strong>Stash and Go:</strong> Drop your items, let go of the mouse, find your target window, and drag them out. The shelf disappears
            automatically.
          </li>
        </ul>

        <h2>Stop Waiting, Start Moving</h2>
        <p>
          You shouldn&apos;t have to pay to remove artificial wait timers just to move your files around efficiently. Holdem is built by developers,
          for productivity enthusiasts, and it will always remain free.
        </p>
        <p>Ready to fix your drag-and-drop workflow?</p>

        <div className="not-prose mt-6 flex flex-wrap gap-3">
          <Link
            href="/"
            className="rounded-lg bg-zinc-900 px-5 py-2.5 text-sm font-medium text-white hover:bg-zinc-800 dark:bg-white dark:text-zinc-900"
          >
            Download Holdem for Free
          </Link>
          <a
            href="https://github.com/iamzubin/holdem"
            target="_blank"
            rel="noopener noreferrer"
            className="rounded-lg border border-zinc-200 px-5 py-2.5 text-sm font-medium hover:bg-zinc-50 dark:border-zinc-700 dark:hover:bg-zinc-900"
          >
            View on GitHub
          </a>
        </div>

        <hr />
        <p className="text-sm text-zinc-500">
          Also comparing <Link href="/vs/dropover">Holdem vs Dropover</Link> • <Link href="/">Go to homepage →</Link>
        </p>
      </article>
    </main>
  )
}
