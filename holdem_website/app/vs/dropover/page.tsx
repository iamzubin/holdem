import type { Metadata } from 'next'
import Link from 'next/link'
import { ButtonLink } from '../../components/ui/button'
import { DOWNLOAD_URL_FALLBACK } from '@/lib/constants'

export const metadata: Metadata = {
  title: 'Holdem vs Dropover: Free Dropover Alternative for Windows',
  description:
    'Dropover is Mac-only. Holdem is the free, open-source Dropover alternative for Windows — shake-to-summon shelf, hotkey, web images, tray app. Full feature, pricing & platform comparison.',
  keywords: [
    'Holdem vs Dropover',
    'Dropover Windows alternative',
    'free Dropover alternative',
    'Dropover for Windows',
    'file shelf Windows',
  ],
  alternates: {
    canonical: 'https://holdem.iamzub.in/vs/dropover',
  },
  openGraph: {
    title: 'Holdem vs Dropover: Free Dropover Alternative for Windows',
    description:
      'Dropover is Mac-only. Holdem brings the same shelf workflow to Windows — free, open-source, no paywall.',
    url: 'https://holdem.iamzub.in/vs/dropover',
    images: [
      {
        url: '/og.png',
        width: 1280,
        height: 640,
        alt: 'Holdem vs Dropover comparison — free file shelf for Windows',
      },
    ],
    type: 'article',
  },
  twitter: {
    card: 'summary_large_image',
    site: '@iamzub_in',
    creator: '@iamzub_in',
    title: 'Holdem vs Dropover: Free Dropover Alternative for Windows',
    description: 'Dropover is Mac-only. Holdem is the free Windows equivalent.',
    images: ['/og.png'],
  },
}

const faqs = [
  {
    q: 'Is there a Dropover for Windows?',
    a: 'No. Per its FAQ, Dropover is exclusively available through the Mac App Store (macOS 13+). Holdem is the closest free, open-source equivalent built natively for Windows 10 and 11.',
  },
  {
    q: 'Is Holdem really free compared to Dropover?',
    a: 'Yes. Dropover is a free download with a 14-day full trial; after the trial a 3-second wait applies unless you buy Dropover Pro ($6.99 one-time). Holdem is MIT-licensed open source: free forever, no wait timers, no pro tier.',
  },
  {
    q: 'Does Holdem work like Dropover’s shelf?',
    a: 'The core workflow is the same: shake your pointer while dragging (or press a hotkey) to summon a floating shelf, stash files, images and folders, navigate freely, then drag them out. Dropover adds Mac-specific extras — notch drops, menu-bar access, pinned shelves, Instant Actions — that Holdem does not replicate.',
  },
  {
    q: 'Can I drag browser images into Holdem like Dropover?',
    a: 'Yes. Holdem already supports dragging web images directly from the browser into the shelf. URL drops and text snippets are on the roadmap; Dropover already supports links and text today.',
  },
  {
    q: 'Does Holdem upload files to the cloud like Dropover?',
    a: 'No. Dropover offers one-click uploads via its free anonymous Dropover Cloud plus iCloud Drive, S3, Google Drive, OneDrive, Imgur, and Dropbox. Holdem is a local-only shelf with no cloud or account of any kind.',
  },
  {
    q: 'Which should I choose?',
    a: 'If you use a Mac, Dropover is the natural choice — it is deeply integrated with macOS. If you use Windows, Dropover is not an option at all, and Holdem gives you the same stash-and-go shelf workflow free and open-source.',
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

export default function VsDropoverPage() {
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
        <p className="text-sm font-medium uppercase tracking-widest text-zinc-500">Compare • Holdem vs Dropover</p>
        <h1 className="!mb-3 text-3xl font-bold md:text-4xl">Holdem vs Dropover: Free Dropover Alternative for Windows</h1>
        <p className="lead text-zinc-600 dark:text-zinc-400">
          Dropover pioneered the temporary file-shelf idea on the Mac — but there is
          no Windows version, and there likely never will be: its FAQ states it is
          exclusively available through the Mac App Store. If you searched “Dropover
          for Windows”, Holdem is the answer: the same shake-to-summon shelf workflow,
          rebuilt free and open-source for Windows 10 and 11.
        </p>

        <div className="not-prose rounded-xl border border-zinc-200 bg-zinc-50 p-4 text-sm dark:border-zinc-800 dark:bg-zinc-900">
          <strong>Verdict:</strong> On a Mac, Dropover wins — it is the native,
          full-featured original. On Windows, Dropover is not an option, and Holdem
          is the free, open-source shelf that covers the core workflow: shake, stash,
          navigate, drop. No trial countdown, no 3-second wait, no $6.99 unlock.
        </div>

        <h2>Holdem vs Dropover: side-by-side</h2>
        <div className="not-prose overflow-x-auto rounded-xl border border-zinc-200 dark:border-zinc-800">
          <table className="w-full text-left text-sm">
            <thead className="bg-zinc-50 text-zinc-500 dark:bg-zinc-900">
              <tr>
                <th className="px-4 py-3 font-medium">Feature</th>
                <th className="px-4 py-3 font-medium">Holdem</th>
                <th className="px-4 py-3 font-medium">Dropover</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-zinc-200 dark:divide-zinc-800">
              <tr>
                <td className="px-4 py-3 font-medium">Platform</td>
                <td className="px-4 py-3">Windows 10/11 x64 (Tauri/Rust)</td>
                <td className="px-4 py-3">macOS 13+ only, Mac App Store exclusive</td>
              </tr>
              <tr>
                <td className="px-4 py-3 font-medium">Pricing</td>
                <td className="px-4 py-3">100% free &amp; open-source (MIT)</td>
                <td className="px-4 py-3">Free download; Dropover Pro $6.99 one-time in-app purchase</td>
              </tr>
              <tr>
                <td className="px-4 py-3 font-medium">Free-tier limits</td>
                <td className="px-4 py-3">None — no timers, no locked features</td>
                <td className="px-4 py-3">14-day full trial, then a 3-second wait before interacting with the shelf</td>
              </tr>
              <tr>
                <td className="px-4 py-3 font-medium">Trigger</td>
                <td className="px-4 py-3">Mouse shake or global hotkey; shelf appears at cursor</td>
                <td className="px-4 py-3">Pointer shake, customizable shortcut, menu bar, MacBook notch, Command Bar</td>
              </tr>
              <tr>
                <td className="px-4 py-3 font-medium">Supported content</td>
                <td className="px-4 py-3">Files, folders, web images (URLs &amp; text on the roadmap)</td>
                <td className="px-4 py-3">Files, folders, documents, images, URLs, text snippets, browser images</td>
              </tr>
              <tr>
                <td className="px-4 py-3 font-medium">Shelf management</td>
                <td className="px-4 py-3">Single floating shelf, draggable, auto-hides when empty</td>
                <td className="px-4 py-3">Pinned shelves, custom titles, color-coding, dockable shelves, detail view, rename/reorder on shelf</td>
              </tr>
              <tr>
                <td className="px-4 py-3 font-medium">Sharing &amp; cloud</td>
                <td className="px-4 py-3">None — local-only, no account</td>
                <td className="px-4 py-3">One-click uploads: free anonymous Dropover Cloud + iCloud Drive, AWS S3, Google Drive, OneDrive, Imgur, Dropbox; macOS Share menu (Mail, Messages, AirDrop)</td>
              </tr>
              <tr>
                <td className="px-4 py-3 font-medium">Automation</td>
                <td className="px-4 py-3">None</td>
                <td className="px-4 py-3">Instant Actions, custom actions &amp; scripts (AppleScript, Automator, UNIX), folder monitoring, screenshot shelves, Shortcuts integration</td>
              </tr>
              <tr>
                <td className="px-4 py-3 font-medium">Footprint</td>
                <td className="px-4 py-3">Tauri + Rust — small installer, quiet tray app, auto-launch</td>
                <td className="px-4 py-3">Native Mac app, frequently updated (5.x line)</td>
              </tr>
            </tbody>
          </table>
        </div>

        <h2>Platform: the whole argument in one row</h2>
        <p>
          This comparison is unusual because the two apps do not compete on the same
          computers. Dropover requires macOS 13 or later and is distributed only via
          the Mac App Store — its own FAQ confirms there is no other way to get it.
          Holdem runs on Windows 10 and 11 (x64) as a small Tauri/Rust tray app. So
          the practical advice is simple: Mac users should evaluate Dropover (and
          Yoink); Windows users choosing between shelves are really choosing between
          Holdem, Dropshelf, and DropPoint — see our{' '}
          <Link href="/vs">full comparison index</Link>.
        </p>

        <h2>Pricing: $0 forever vs $6.99 once</h2>
        <p>
          Dropover follows the classic indie-Mac model: the download is free and
          includes a fully functional 14-day trial. After the trial expires you are
          not charged — the app keeps working but imposes a 3-second wait before you
          can interact with the shelf. Removing it costs $6.99 as a one-time
          in-app purchase (per its Mac App Store listing; verify the current price
          before buying). That is fair pricing for a mature app — but it is still a
          paywall on the core interaction. Holdem has no trial, no timer, and no
          paid tier at all: the MIT-licensed code is on{' '}
          <a href="https://github.com/iamzubin/holdem" target="_blank" rel="noopener noreferrer">GitHub</a>,
          and every feature ships to every user. See <Link href="/pricing">pricing</Link> for
          the full price table.
        </p>

        <h2>Trigger and shelf UX</h2>
        <p>
          Both apps share the signature gesture: shake the pointer while dragging and
          a shelf pops up. Dropover layers on more entry points — a customizable
          keyboard shortcut, a menu-bar control, dropping onto the MacBook notch,
          Control Center controls, and a Command Bar — plus shelf furniture Holdem
          deliberately skips: pinned shelves with custom titles, color-coding,
          docking to screen edges, and a detail view where you can preview, rename,
          and reorder stashed files. Holdem does one thing: the shelf appears exactly
          where your cursor is, you drop files in, you navigate anywhere without
          holding the mouse button, you drag them out, and the shelf auto-hides. If
          you want a shelf that doubles as a workspace, Dropover is richer. If you
          want the fastest possible stash-and-go on Windows, Holdem&apos;s minimalism
          is the point.
        </p>

        <h2>Content, sharing, and automation</h2>
        <p>
          Dropover handles anything draggable on macOS — files, folders, images,
          links, text — and then lets you <em>do things</em> with the stash: share
          via AirDrop/Mail/Messages, resize images, extract text, zip archives, run
          Instant Actions, fire custom scripts, watch folders, and upload to seven
          cloud services including its own free anonymous Dropover Cloud. Holdem
          covers files, folders, and web images dragged from the browser; URL and
          text drops are roadmap items, and there is no cloud, sharing, or scripting
          layer — by design it is a local-only utility with no account and no
          network calls. Choose based on whether your shelf is a transit lounge
          (Holdem) or a workbench (Dropover).
        </p>

        <h2>How to switch: Dropover habits → Holdem</h2>
        <ol>
          <li><Link href="/download">Download Holdem for Windows</Link> (free .exe installer).</li>
          <li>Drag files → shake the mouse → the shelf appears at your cursor.</li>
          <li>Release the mouse, switch desktops and apps freely.</li>
          <li>Drag items out to the destination — the shelf auto-hides when empty.</li>
        </ol>
        <p>
          Full walkthrough with tray settings and hotkeys: see the{' '}
          <Link href="/docs">docs</Link>.
        </p>

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

        <h2>FAQ: Dropover on Windows</h2>
        {faqs.map((f) => (
          <div key={f.q}>
            <h3>{f.q}</h3>
            <p>{f.a}</p>
          </div>
        ))}

        <h2>Sources</h2>
        <ul>
          <li><a href="https://dropoverapp.com/" target="_blank" rel="noopener noreferrer">Dropover homepage</a> — triggers, content, actions, cloud integrations</li>
          <li><a href="https://dropoverapp.com/faq" target="_blank" rel="noopener noreferrer">Dropover FAQ</a> — 14-day trial, 3-second wait, Mac App Store exclusivity</li>
          <li><a href="https://apps.apple.com/us/app/dropover-easier-drag-drop/id1355679052" target="_blank" rel="noopener noreferrer">Dropover on the Mac App Store</a> — $6.99 Pro, macOS 13+ requirement</li>
        </ul>

        <hr />
        <p className="text-sm text-zinc-500">
          Also: <Link href="/vs/yoink">Holdem vs Yoink</Link> •{' '}
          <Link href="/vs/dropshelf">Holdem vs Dropshelf</Link> •{' '}
          <Link href="/vs/droppoint">Holdem vs DropPoint</Link> •{' '}
          <Link href="/">Go to homepage →</Link>
        </p>
      </article>
    </main>
  )
}
