import type { Metadata } from 'next'
import Link from 'next/link'
import { DOWNLOAD_URL_FALLBACK } from '@/lib/constants'

export const metadata: Metadata = {
  title: 'Holdem vs Dropshelf: The Best Free Dropshelf Alternative for Windows',
  description:
    'Holdem vs Dropshelf — 100% free, open-source drag-and-drop shelf for Windows with no 3-second wait timers. Full feature, pricing & design comparison.',
  keywords: [
    'Holdem vs Dropshelf',
    'Dropshelf alternative',
    'free Dropshelf alternative',
    'Dropshelf Windows',
    'Dropshelf Pro vs free',
  ],
  alternates: {
    canonical: 'https://holdem.iamzub.in/vs/dropshelf',
  },
  openGraph: {
    title: 'Holdem vs Dropshelf: The Best Free Dropshelf Alternative for Windows',
    description:
      '100% free, open-source shelf for Windows — no 3-second wait. Full Holdem vs Dropshelf comparison.',
    url: 'https://holdem.iamzub.in/vs/dropshelf',
    images: [
      {
        url: '/og.png',
        width: 1280,
        height: 640,
        alt: 'Holdem vs Dropshelf comparison — free file shelf for Windows',
      },
    ],
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

const faqs = [
  {
    q: 'What is the Dropshelf 3-second wait?',
    a: 'Per its Microsoft Store listing, the free version of Dropshelf forces a 3-second wait before you can add more items to a shelf. Dropshelf Pro, a one-time in-app purchase, removes the wait and unlocks Pro features. Holdem has no wait timers at all.',
  },
  {
    q: 'Is Holdem really a free Dropshelf alternative?',
    a: 'Yes for the core workflow: shake-to-summon shelf, stash files, drop anywhere. Holdem is MIT-licensed and free forever. Dropshelf is richer (saved/named/colored shelves, URLs, text) but its free tier throttles you and its extras sit behind Pro.',
  },
  {
    q: 'What can Dropshelf hold that Holdem cannot?',
    a: 'Today: URLs, text snippets, and multimedia beyond files and web images — plus saved shelves you can name, color-code (Pro), and reopen later from the tray. Holdem holds files, folders, and browser images; URL/text drops are on the roadmap.',
  },
  {
    q: 'How do the triggers compare?',
    a: 'Both summon with a mouse shake. Dropshelf also offers Win+Shift+Z for a new shelf and Win+Shift+X to create one from clipboard contents, with adjustable shake sensitivity and per-app exclusions. Holdem offers shake plus a customizable global hotkey.',
  },
  {
    q: 'Which looks more native on Windows 11?',
    a: 'Dropshelf is built with WinUI 3 and Fluent acrylic materials, so it looks like a first-party Windows 11 app. Holdem goes for a minimal dark floating shelf. Taste differs — try both; both are small downloads.',
  },
  {
    q: 'Should I pay for Dropshelf Pro or use Holdem?',
    a: 'If you live in your shelf all day and want saved, named, color-coded shelves with zero waiting, check the Pro price on the Microsoft Store listing — it is a one-time purchase. If you want the core stash-and-go workflow free forever, Holdem covers it.',
  },
]

const faqSchema = {
  '@context': 'https://schema.org',
  '@type': 'FAQPage',
  mainEntity: faqs.map((f) => ({
    '@type': 'Question',
    name: f.q,
    acceptedAnswer: { '@type': 'Answer', text: f.a },
  })),
}

export default function VsDropshelfPage() {
  return (
    <main className="mx-auto max-w-3xl py-8">
      <script
        type="application/ld+json"
        dangerouslySetInnerHTML={{ __html: JSON.stringify(faqSchema) }}
      />
      <Link href="/" className="text-sm text-zinc-500 hover:text-zinc-900 dark:hover:text-zinc-100">
        ← Back to Holdem
      </Link>
      <article className="prose prose-zinc mt-6 max-w-none dark:prose-invert prose-headings:tracking-tight prose-a:text-zinc-900 dark:prose-a:text-zinc-100">
        <p className="text-sm font-medium uppercase tracking-widest text-zinc-500">Compare • Holdem vs Dropshelf</p>
        <h1 className="!mb-3 text-3xl font-bold md:text-4xl">Holdem vs Dropshelf: The Best Free Dropshelf Alternative for Windows</h1>
        <p className="lead text-zinc-600 dark:text-zinc-400">
          Dropshelf is the most polished shelf built specifically for Windows — a
          WinUI 3 app with saved shelves, list/grid views, and URL support. Its
          catch is the business model: the free tier makes you wait 3 seconds every
          time you add items, and the good stuff is Pro. Holdem is the free,
          open-source answer to exactly that trade-off.
        </p>

        <div className="not-prose rounded-xl border border-zinc-200 bg-zinc-50 p-4 text-sm dark:border-zinc-800 dark:bg-zinc-900">
          <strong>Verdict:</strong> Power users who want saved, named, color-coded
          shelves should price-check Dropshelf Pro (one-time purchase). Everyone
          else — anyone who just wants to shake, stash, and drop without waiting —
          gets that free forever with Holdem.
        </div>

        <h2>Holdem vs Dropshelf: side-by-side</h2>
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
                <td className="px-4 py-3">100% free &amp; open-source (MIT)</td>
                <td className="px-4 py-3">Free with 3-second wait; Pro is a one-time in-app purchase (see Store listing for current price)</td>
              </tr>
              <tr>
                <td className="px-4 py-3 font-medium">Wait timers</td>
                <td className="px-4 py-3">None. Completely frictionless.</td>
                <td className="px-4 py-3">3-second forced wait on the free tier before adding more items</td>
              </tr>
              <tr>
                <td className="px-4 py-3 font-medium">Trigger</td>
                <td className="px-4 py-3">Mouse shake or global hotkey at cursor</td>
                <td className="px-4 py-3">Mouse shake (adjustable sensitivity) or Win+Shift+Z; Win+Shift+X creates a shelf from clipboard; per-app exclusions</td>
              </tr>
              <tr>
                <td className="px-4 py-3 font-medium">Supported content</td>
                <td className="px-4 py-3">Files, folders, web images (URLs &amp; text on the roadmap)</td>
                <td className="px-4 py-3">Files, folders, documents, images, videos, multimedia, URLs, web images, text snippets</td>
              </tr>
              <tr>
                <td className="px-4 py-3 font-medium">Shelf views</td>
                <td className="px-4 py-3">Single minimal floating shelf</td>
                <td className="px-4 py-3">List or grid layout; movable; Ctrl-drag copies (leaves originals); web-link favicon previews</td>
              </tr>
              <tr>
                <td className="px-4 py-3 font-medium">Save for later</td>
                <td className="px-4 py-3">No — shelf auto-hides when empty</td>
                <td className="px-4 py-3">Closing a non-empty shelf auto-saves it; reopen timestamped shelves from the tray; custom names + colors are Pro features</td>
              </tr>
              <tr>
                <td className="px-4 py-3 font-medium">Design</td>
                <td className="px-4 py-3">Minimal dark floating shelf (Tauri)</td>
                <td className="px-4 py-3">Native Windows 11 look — WinUI 3, Fluent acrylic, monochrome tray-icon option</td>
              </tr>
              <tr>
                <td className="px-4 py-3 font-medium">Scale</td>
                <td className="px-4 py-3">Lightweight tray app</td>
                <td className="px-4 py-3">Reworked file handling for 100+ item drops without hanging (per release notes)</td>
              </tr>
              <tr>
                <td className="px-4 py-3 font-medium">Philosophy</td>
                <td className="px-4 py-3">Speed, open source, zero paywalls</td>
                <td className="px-4 py-3">Native Windows craft, freemium with paid Pro tier</td>
              </tr>
            </tbody>
          </table>
        </div>

        <h2>The 3-second wait is the whole story</h2>
        <p>
          Dropshelf&apos;s core interaction is genuinely good: drag files, shake,
          drop them on the shelf beside your cursor, go about your business, drag
          them out. The free version includes all of that — with one catch spelled
          out on its Store page: a forced 3-second wait before you can add more
          items to a shelf. Used once a day, you will barely notice. Used fifty
          times a day, in flow state, artificial delays are maddening — and that is
          precisely the moment you are asked to pay. Holdem exists because that
          trade-off should not exist: the same shake-to-summon interaction with zero
          waiting, zero tiers, zero checkout. That is also why this page can be
          blunt about Dropshelf&apos;s strengths below — when the core move is free
          everywhere, the comparison becomes about features, not tolls.
        </p>

        <h2>Where Dropshelf is honestly ahead</h2>
        <p>
          Credit where due: Dropshelf is the richer app today. It accepts URLs, text
          snippets, and multimedia — content types Holdem has on its roadmap but
          does not ship yet. Its shelves can be viewed as lists or grids, moved
          anywhere, and — the killer feature — automatically saved when closed with
          items inside, then reopened from the tray with timestamps, custom names,
          and colors (naming/colors are Pro). It handles 100+ file drops without
          hanging, shows favicons for web links, lets you Ctrl-drag to copy instead
          of move, and its WinUI 3 interface with Fluent acrylic genuinely looks at
          home on Windows 11. Settings go deep: shake sensitivity, excluded apps,
          shelf placement, auto-close behavior, monochrome tray icon. If your shelf
          is a project inbox you return to for days, Dropshelf earns its Pro price.
        </p>

        <h2>Where Holdem wins</h2>
        <p>
          Holdem wins on three things: price ($0, forever, no asterisk), friction
          (no timers, shelf appears at the cursor and vanishes when empty), and
          footprint (a Tauri/Rust tray app that sips resources and auto-launches).
          It is also fully open-source under MIT — you can read every line, while
          Dropshelf&apos;s public repo is issue-only. For the canonical use case —
          “I&apos;m mid-drag and need somewhere to park this for thirty seconds” —
          Holdem is complete. Browser-image drops already work, so research-style
          collect-and-drop flows are covered; only URL/text stashes and multi-shelf
          organization await the roadmap.
        </p>

        <h2>Switching: Dropshelf habits → Holdem</h2>
        <ol>
          <li><Link href="/download">Download Holdem for Windows</Link> (free .exe installer).</li>
          <li>Drag files → shake the mouse → the shelf appears at your cursor. No 3-second wait — ever.</li>
          <li>Release the mouse, switch folders, apps, and desktops freely.</li>
          <li>Drag items out to the destination — the shelf auto-hides when empty.</li>
        </ol>

        <div className="not-prose mt-6 flex flex-wrap gap-3">
          <a
            href={DOWNLOAD_URL_FALLBACK}
            download
            className="rounded-lg bg-zinc-900 px-5 py-2.5 text-sm font-medium text-white hover:bg-zinc-800 dark:bg-white dark:text-zinc-900"
          >
            Download Holdem for Free
          </a>
          <a
            href="https://github.com/iamzubin/holdem"
            target="_blank"
            rel="noopener noreferrer"
            className="rounded-lg border border-zinc-200 px-5 py-2.5 text-sm font-medium hover:bg-zinc-50 dark:border-zinc-700 dark:hover:bg-zinc-900"
          >
            View on GitHub
          </a>
        </div>

        <h2>FAQ: Holdem vs Dropshelf</h2>
        {faqs.map((f) => (
          <div key={f.q}>
            <h3>{f.q}</h3>
            <p>{f.a}</p>
          </div>
        ))}

        <h2>Sources</h2>
        <ul>
          <li><a href="https://apps.microsoft.com/detail/9mzpc6p14l7n" target="_blank" rel="noopener noreferrer">Dropshelf on the Microsoft Store</a> — free tier 3-second wait, Pro in-app purchase, WinUI 3, content support</li>
          <li><a href="https://github.com/williamckha/dropshelf-repo" target="_blank" rel="noopener noreferrer">Dropshelf issue repo &amp; releases</a> — Windows App SDK/WinUI 3, performance rework, shelf features</li>
          <li><a href="https://www.makeuseof.com/tiny-windows-app-makes-moving-files-between-apps-easier/" target="_blank" rel="noopener noreferrer">MakeUseOf hands-on</a> — Win+Shift+Z, saved shelves, Pro name/color, clipboard shortcut</li>
          <li><a href="https://techpp.com/2025/04/08/dropshelf-drag-and-drop-on-windows/" target="_blank" rel="noopener noreferrer">TechPP overview</a> — shelf behavior, free-tier wait, Pro features</li>
        </ul>

        <hr />
        <p className="text-sm text-zinc-500">
          Also: <Link href="/vs/dropover">Holdem vs Dropover</Link> •{' '}
          <Link href="/vs/yoink">Holdem vs Yoink</Link> •{' '}
          <Link href="/vs/droppoint">Holdem vs DropPoint</Link> •{' '}
          <Link href="/">Go to homepage →</Link>
        </p>
      </article>
    </main>
  )
}
