import type { Metadata } from 'next'
import Link from 'next/link'

export const metadata: Metadata = {
  title: 'Holdem vs DropPoint: The Best Free Dropover Alternative for Windows',
  description:
    'Holdem vs DropPoint — Tauri/Rust vs Electron, minimal hyper-functional design vs dated UI, mouse-shake vs Shift+CapsLock. See why Holdem is the modern upgrade.',
  openGraph: {
    title: 'Holdem vs DropPoint: The Best Free Dropover Alternative for Windows',
    description: 'Tauri/Rust vs Electron — clean minimal vs dated UI. See Holdem vs DropPoint.',
    url: 'https://holdem.iamzub.in/vs/droppoint',
    images: [{ url: '/og.png', width: 1280, height: 640, alt: 'Holdem vs DropPoint' }],
    type: 'article',
  },
  twitter: {
    card: 'summary_large_image',
    site: '@iamzub_in',
    creator: '@iamzub_in',
    title: 'Holdem vs DropPoint: The Best Free Dropover Alternative for Windows',
    description: 'Tauri/Rust vs Electron — clean minimal vs dated UI.',
    images: ['/og.png'],
  },
}

export default function VsDropointPage() {
  return (
    <main className="mx-auto max-w-3xl py-8">
      <Link href="/" className="text-sm text-zinc-500 hover:text-zinc-900 dark:hover:text-zinc-100">
        ← Back to Holdem
      </Link>

      <article className="prose prose-zinc mt-6 max-w-none dark:prose-invert prose-headings:tracking-tight prose-a:text-zinc-900 dark:prose-a:text-zinc-100">
        <p className="text-sm font-medium uppercase tracking-widest text-zinc-500">Compare • Holdem vs DropPoint</p>
        <h1 className="!mb-3 text-3xl font-bold md:text-4xl">
          The Best Free Dropover Alternative for Windows: Holdem vs. DropPoint
        </h1>
        <p className="lead text-zinc-600 dark:text-zinc-400">
          If you work with multiple monitors or frequently move files across crowded desktops, you already know how painful Windows drag-and-drop can be.
        </p>

        <p>
          You click a file, drag it to the edge of your screen, clumsily <code>Alt + Tab</code> to find your target window, and pray you don&apos;t
          drop it in the wrong place.
        </p>
        <p>
          Mac users have long relied on tools like <strong>Dropover</strong> to solve this by creating a temporary &quot;shelf&quot; to hold files
          mid-transit. If you&apos;re searching for a <strong>free Dropover alternative for Windows</strong>, you will likely come across two popular
          open-source options: <strong>DropPoint</strong> and <strong>Holdem</strong>.
        </p>
        <p>Both are free, but they take completely different approaches to performance, user experience, and design. Here is why Holdem is the modern upgrade your workflow needs.</p>

        <h2>Holdem vs. DropPoint: The Core Differences</h2>
        <p>Here is a quick look at how the two drag-and-drop utilities compare:</p>

        <div className="not-prose overflow-x-auto rounded-xl border border-zinc-200 dark:border-zinc-800">
          <table className="w-full text-left text-sm">
            <thead className="bg-zinc-50 text-zinc-500 dark:bg-zinc-900">
              <tr>
                <th className="px-4 py-3 font-medium">Feature</th>
                <th className="px-4 py-3 font-medium">Holdem</th>
                <th className="px-4 py-3 font-medium">DropPoint</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-zinc-200 dark:divide-zinc-800">
              <tr>
                <td className="px-4 py-3 font-medium">Tech Stack</td>
                <td className="px-4 py-3">Tauri, Rust, TypeScript (Extremely lightweight)</td>
                <td className="px-4 py-3">Electron, Node.js (High memory usage)</td>
              </tr>
              <tr>
                <td className="px-4 py-3 font-medium">UI &amp; Aesthetics</td>
                <td className="px-4 py-3">Clean, minimal, and hyper-functional</td>
                <td className="px-4 py-3">Dated, clunky, and intrusive</td>
              </tr>
              <tr>
                <td className="px-4 py-3 font-medium">Trigger Mechanism</td>
                <td className="px-4 py-3">Mouse shake or Global Hotkey</td>
                <td className="px-4 py-3">Global Hotkey (<code>Shift + Caps Lock</code>) or Tray icon</td>
              </tr>
              <tr>
                <td className="px-4 py-3 font-medium">Supported Content</td>
                <td className="px-4 py-3">Files, folders, and web images (URLs coming soon)</td>
                <td className="px-4 py-3">Files and folders</td>
              </tr>
              <tr>
                <td className="px-4 py-3 font-medium">Performance</td>
                <td className="px-4 py-3">Instantaneous, low resource footprint</td>
                <td className="px-4 py-3">Heavier background process</td>
              </tr>
            </tbody>
          </table>
        </div>

        <h2>Why Holdem Wins for Pure Productivity</h2>
        <p>
          While DropPoint is a great project that helped pave the way for Windows drag-and-drop utilities, its underlying technology and interface hold
          it back. DropPoint is built on Electron — the same heavy framework that runs apps like Slack and Spotify — and, let&apos;s be honest, the UI
          looks like it belongs in a different decade.
        </p>
        <p>Holdem was built from the ground up to fix this.</p>

        <h3>1. Built on Rust and Tauri for Speed</h3>
        <p>Holdem uses Tauri and Rust instead of Electron. This means it has a fraction of the memory footprint, uses almost zero background CPU, and feels incredibly snappy.</p>

        <h3>2. Design That Doesn&apos;t Hurt to Look At</h3>
        <p>
          Utility apps don&apos;t have to be ugly. Holdem features a brutalist, hyper-engineered, and minimal interface. It looks modern, acts
          predictably, and gets out of your way instead of cluttering your screen with a clunky UI.
        </p>

        <h3>3. Frictionless &quot;Mouse Shake&quot; Activation</h3>
        <p>
          With DropPoint, you have to press an awkward keyboard shortcut (<code>Shift + Caps Lock</code>) every time you want to summon the shelf.
          Holdem is completely frictionless. Simply <strong>shake your mouse</strong>, and the digital basket appears exactly where your cursor is.
        </p>

        <h3>4. Built for the Modern Web</h3>
        <p>
          Holdem goes beyond just local files. You can already drag and drop web images directly from your browser onto the shelf, and support for
          URLs is actively in development. Drop your items, navigate to your target window, and drag them out. The shelf disappears automatically when
          you&apos;re done.
        </p>

        <h2>Stop Fighting with Windows Drag-and-Drop</h2>
        <p>
          You shouldn&apos;t have to sacrifice system memory or look at an ugly interface just to move your files around efficiently. Holdem is built
          for productivity enthusiasts — it is lightning-fast, entirely open-source, and free forever.
        </p>
        <p>Ready to fix your workflow?</p>

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
          More comparisons: <Link href="/vs/dropshelf">Holdem vs Dropshelf</Link> • <Link href="/vs/dropover">Holdem vs Dropover</Link> •{' '}
          <Link href="/">Go to homepage →</Link>
        </p>
      </article>
    </main>
  )
}
