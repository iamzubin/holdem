import type { Metadata } from 'next'
import Link from 'next/link'
import { ButtonLink } from '../../components/ui/button'
import { DOWNLOAD_URL_FALLBACK } from '@/lib/constants'

export const metadata: Metadata = {
  title: 'Holdem vs Yoink: Free Yoink Alternative for Windows',
  description:
    'Yoink is Mac-only ($8.99). Holdem is the free, open-source Yoink alternative for Windows — shake-to-summon shelf, hotkey, web images, tray app. Full feature, pricing & platform comparison.',
  keywords: [
    'Holdem vs Yoink',
    'Yoink Windows alternative',
    'free Yoink alternative',
    'Yoink for Windows',
    'file shelf Windows',
  ],
  alternates: {
    canonical: 'https://holdem.iamzub.in/vs/yoink',
  },
  openGraph: {
    title: 'Holdem vs Yoink: Free Yoink Alternative for Windows',
    description:
      'Yoink is Mac-only ($8.99). Holdem brings the same shelf workflow to Windows — free, open-source, no paywall.',
    url: 'https://holdem.iamzub.in/vs/yoink',
    images: [
      {
        url: '/og.png',
        width: 1280,
        height: 640,
        alt: 'Holdem vs Yoink comparison — free file shelf for Windows',
      },
    ],
    type: 'article',
  },
  twitter: {
    card: 'summary_large_image',
    site: '@iamzub_in',
    creator: '@iamzub_in',
    title: 'Holdem vs Yoink: Free Yoink Alternative for Windows',
    description: 'Yoink is Mac-only. Holdem is the free Windows equivalent.',
    images: ['/og.png'],
  },
}

const faqs = [
  {
    q: 'Is there a Yoink for Windows?',
    a: 'No. Yoink by Eternal Storms is macOS-only (a separate Yoink app for iPhone/iPad is sold separately). Holdem is the closest free, open-source equivalent built natively for Windows 10 and 11.',
  },
  {
    q: 'How much does Yoink cost vs Holdem?',
    a: 'Per the developer site and Mac App Store, Yoink is a $8.99 (US) one-time purchase — also available via Setapp — with a free trial on the developer website. Holdem is MIT-licensed open source: free forever with no trial or paywall.',
  },
  {
    q: 'Does Holdem work like Yoink’s shelf?',
    a: 'The core workflow matches: Yoink shows a shelf at the screen edge (or cursor) when you start dragging; Holdem shows a floating shelf at your cursor when you shake the mouse while dragging or press the global hotkey. Stash files, navigate freely, drag them out.',
  },
  {
    q: 'Does Holdem have Yoink’s clipboard history and Handoff?',
    a: 'No. Yoink’s clipboard history (with Notification Center widget), QuickLook previews, file Stacks, and Handoff/Continuity Camera transfers with iOS devices are Mac-ecosystem features Holdem does not replicate. Holdem focuses on the core Windows shelf: files, folders, and browser images.',
  },
  {
    q: 'Which is lighter: Holdem or Yoink?',
    a: 'Both are light native apps on their own platforms — Yoink is a long-optimized native Mac app, Holdem is a small Tauri/Rust tray app for Windows. Neither is an Electron app, so neither carries Electron’s memory overhead (unlike DropPoint).',
  },
  {
    q: 'Which should I choose?',
    a: 'If you use a Mac, Yoink is the proven classic — a decade-plus of refinement, deep OS integration, one-time price. If you use Windows, Yoink is not an option and Holdem gives you the same stash-and-go workflow free and open-source.',
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

export default function VsYoinkPage() {
  return (
    <main className="mx-auto max-w-3xl py-8">
      <script
        type="application/ld+json"
        dangerouslySetInnerHTML={{ __html: JSON.stringify(faqSchema) }}
      />
      <Link
        href="/"
        className="text-sm text-zinc-500 hover:text-zinc-900 dark:hover:text-zinc-100"
      >
        ← Back to Holdem
      </Link>
      <article className="prose prose-zinc dark:prose-invert prose-headings:tracking-tight prose-a:text-zinc-900 dark:prose-a:text-zinc-100 mt-6 max-w-none">
        <p className="text-sm font-medium tracking-widest text-zinc-500 uppercase">
          Compare • Holdem vs Yoink
        </p>
        <h1 className="!mb-3 text-3xl font-bold md:text-4xl">
          Holdem vs Yoink: Free Yoink Alternative for Windows
        </h1>
        <p className="lead text-zinc-600 dark:text-zinc-400">
          Yoink is the classic Mac drag-and-drop shelf — refined over many major
          versions, with a loyal following and deep macOS integration. But it
          has never had a Windows version. If you searched “Yoink for Windows”,
          Holdem is the answer: the same stash-and-go shelf workflow, rebuilt
          free and open-source for Windows 10 and 11.
        </p>

        <div className="not-prose rounded-xl border border-zinc-200 bg-zinc-50 p-4 text-sm dark:border-zinc-800 dark:bg-zinc-900">
          <strong>Verdict:</strong> On a Mac, Yoink wins — $8.99 once buys a
          shelf, clipboard history, QuickLook, and Handoff refined over a
          decade. On Windows, Yoink is not an option, and Holdem covers the core
          shelf workflow free: shake, stash, navigate, drop. No purchase, no
          subscription, no trial clock.
        </div>

        <h2>Holdem vs Yoink: side-by-side</h2>
        <div className="not-prose overflow-x-auto rounded-xl border border-zinc-200 dark:border-zinc-800">
          <table className="w-full text-left text-sm">
            <thead className="bg-zinc-50 text-zinc-500 dark:bg-zinc-900">
              <tr>
                <th className="px-4 py-3 font-medium">Feature</th>
                <th className="px-4 py-3 font-medium">Holdem</th>
                <th className="px-4 py-3 font-medium">Yoink</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-zinc-200 dark:divide-zinc-800">
              <tr>
                <td className="px-4 py-3 font-medium">Platform</td>
                <td className="px-4 py-3">Windows 10/11 x64 (Tauri/Rust)</td>
                <td className="px-4 py-3">
                  macOS 10.13+ (native Mac app; separate iOS/iPad app sold
                  separately)
                </td>
              </tr>
              <tr>
                <td className="px-4 py-3 font-medium">Pricing</td>
                <td className="px-4 py-3">100% free &amp; open-source (MIT)</td>
                <td className="px-4 py-3">
                  $8.99 (US) one-time — Mac App Store, direct, or Setapp; free
                  trial on the developer site
                </td>
              </tr>
              <tr>
                <td className="px-4 py-3 font-medium">Trigger</td>
                <td className="px-4 py-3">
                  Mouse shake or global hotkey; shelf appears at cursor
                </td>
                <td className="px-4 py-3">
                  Appears at screen edge (or cursor) when dragging; keyboard
                  shortcut with long-press recall and double-press clipboard
                  save; per-app ignore list
                </td>
              </tr>
              <tr>
                <td className="px-4 py-3 font-medium">Supported content</td>
                <td className="px-4 py-3">
                  Files, folders, web images (URLs &amp; text on the roadmap)
                </td>
                <td className="px-4 py-3">
                  Virtually anything draggable: Finder files, app content, web
                  images, text snippets
                </td>
              </tr>
              <tr>
                <td className="px-4 py-3 font-medium">Move/copy semantics</td>
                <td className="px-4 py-3">Drag out to destination</td>
                <td className="px-4 py-3">
                  Finder-like: ⌥ force-copies, ⌘ force-moves; multiple items
                  auto-stack, splittable in-app
                </td>
              </tr>
              <tr>
                <td className="px-4 py-3 font-medium">
                  Previews &amp; history
                </td>
                <td className="px-4 py-3">None</td>
                <td className="px-4 py-3">
                  Icon + pinnable QuickLook previews; clipboard history via
                  widget with pin/search; password managers excluded by default
                </td>
              </tr>
              <tr>
                <td className="px-4 py-3 font-medium">Cross-device</td>
                <td className="px-4 py-3">None</td>
                <td className="px-4 py-3">
                  Handoff between Macs/iPhones/iPads running Yoink; Continuity
                  Camera import; Share extension, Services, Quick Action,
                  Terminal/Automator support
                </td>
              </tr>
              <tr>
                <td className="px-4 py-3 font-medium">Customization</td>
                <td className="px-4 py-3">
                  Minimal — tray settings, hotkey, auto-launch
                </td>
                <td className="px-4 py-3">
                  Shelf size/position (6 spots), appearance timing, ignored
                  apps, 8+ interface languages
                </td>
              </tr>
              <tr>
                <td className="px-4 py-3 font-medium">Footprint</td>
                <td className="px-4 py-3">
                  Tauri + Rust — small, quiet tray app
                </td>
                <td className="px-4 py-3">
                  Native Mac app, actively maintained (v3.7.x line in 2026)
                </td>
              </tr>
            </tbody>
          </table>
        </div>

        <h2>Platform: why this page exists</h2>
        <p>
          Yoink is developed by Eternal Storms exclusively for Apple platforms:
          it needs macOS 10.13 or newer on the desktop, and the iPhone/iPad
          companion is a separate purchase. There is no Windows build, so
          Windows users searching for it are really asking “what is the Yoink of
          Windows?” Holdem is built to be that answer — not a clone of
          Yoink&apos;s Mac-only extras, but the same fundamental interaction: a
          temporary shelf that frees your mouse hand while you navigate to the
          destination. Mac users comparing shelves should look at Yoink vs
          Dropover instead; Windows users should compare Holdem against{' '}
          <Link href="/vs/dropshelf">Dropshelf</Link> and{' '}
          <Link href="/vs/droppoint">DropPoint</Link>.
        </p>

        <h2>Pricing: $0 forever vs $8.99 once</h2>
        <p>
          Yoink&apos;s pricing is the friendliest kind of commercial software:
          pay $8.99 once (US Mac App Store pricing; also sold direct and bundled
          in Setapp), get free updates, evaluate first with the free trial from
          the developer&apos;s site. No subscription, no per-feature unlocks.
          Still, it is $8.99 plus a Mac. Holdem costs nothing on any Windows PC
          and its MIT-licensed source is public on{' '}
          <a
            href="https://github.com/iamzubin/holdem"
            target="_blank"
            rel="noopener noreferrer"
          >
            GitHub
          </a>
          . Neither app has wait timers or a “Pro” tier gating the core shelf —
          a contrast with the freemium Windows/Mac shelves (see{' '}
          <Link href="/pricing">pricing</Link>).
        </p>

        <h2>Shelf behavior: edge vs cursor</h2>
        <p>
          Yoink&apos;s signature is positional flexibility: the shelf can live
          at the screen edge and slide in as you approach, or pop up beside your
          cursor; you can resize it, park it in one of six positions, and tell
          it to ignore apps where you never need it. Its keyboard shortcut
          doubles as a memory — long-press to recall files you already moved
          out, double-press to stash your clipboard contents. Holdem is simpler:
          one floating shelf, summoned by shaking the mouse mid-drag or by
          hotkey, appearing exactly at the cursor, auto-hiding when empty. Yoink
          behaves like a shelf that lives on your desk; Holdem behaves like a
          shelf that appears only when your hands are full.
        </p>

        <h2>Where Yoink is honestly ahead</h2>
        <p>
          A decade-plus head start shows. Yoink understands Finder semantics
          (Option to copy, Command to move), stacks multiple drops
          automatically, previews anything via QuickLook, keeps a searchable
          clipboard history with pinned items, and moves content across your
          Apple devices with Handoff. It is localized into 8+ languages and
          plugs into Services, Quick Actions, and Automator. Holdem does not
          attempt any of this: it is a young, Windows-only, local-only shelf
          whose roadmap (URL/text drops, multiple shelves) targets the dragging
          workflow specifically. If your computer is a Mac, buy Yoink with
          confidence. If it runs Windows, Holdem is the free way to get the
          shelf habit.
        </p>

        <h2>Switching: Yoink habits → Holdem</h2>
        <ol>
          <li>
            <Link href="/download">Download Holdem for Windows</Link> (free .exe
            installer).
          </li>
          <li>
            Drag files → shake the mouse → the shelf appears at your cursor.
          </li>
          <li>Release the mouse, switch folders, apps, and desktops freely.</li>
          <li>
            Drag items out to the destination — the shelf auto-hides when empty.
          </li>
        </ol>

        <div className="not-prose mt-6 flex flex-wrap gap-3">
          <ButtonLink
            variant="primary"
            size="md"
            className="text-sm"
            href={DOWNLOAD_URL_FALLBACK}
            download
          >
            Download Holdem for Windows — Free
          </ButtonLink>
          <ButtonLink
            variant="secondary"
            size="md"
            className="text-sm"
            href="https://github.com/iamzubin/holdem"
            target="_blank"
            rel="noopener noreferrer"
          >
            View on GitHub
          </ButtonLink>
        </div>

        <h2>FAQ: Yoink on Windows</h2>
        {faqs.map((f) => (
          <div key={f.q}>
            <h3>{f.q}</h3>
            <p>{f.a}</p>
          </div>
        ))}

        <h2>Sources</h2>
        <ul>
          <li>
            <a
              href="https://eternalstorms.at/yoink/mac/"
              target="_blank"
              rel="noopener noreferrer"
            >
              Yoink for Mac homepage
            </a>{' '}
            — $8.99 pricing, trial, triggers, clipboard history, Handoff
          </li>
          <li>
            <a
              href="https://blog.eternalstorms.at/2026/08/03/yoink-v3-7-6-improves-compatibility-with-browsers/"
              target="_blank"
              rel="noopener noreferrer"
            >
              Eternal Storms blog (v3.7.6)
            </a>{' '}
            — one-time pricing, 28-day trial, macOS 10.13+ requirement
          </li>
          <li>
            <a
              href="https://setapp.com/apps/yoink"
              target="_blank"
              rel="noopener noreferrer"
            >
              Yoink on Setapp
            </a>{' '}
            — stacks, shortcuts, Handoff feature rundown
          </li>
        </ul>

        <hr />
        <p className="text-sm text-zinc-500">
          Also: <Link href="/vs/dropover">Holdem vs Dropover</Link> •{' '}
          <Link href="/vs/dropshelf">Holdem vs Dropshelf</Link> •{' '}
          <Link href="/vs/droppoint">Holdem vs DropPoint</Link> •{' '}
          <Link href="/">Go to homepage →</Link>
        </p>
      </article>
    </main>
  )
}
