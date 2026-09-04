import type { Metadata } from 'next'
import Link from 'next/link'
import { ButtonLink } from '../../components/ui/button'
import { DOWNLOAD_URL_FALLBACK } from '@/lib/constants'

export const metadata: Metadata = {
  title: 'Holdem vs DropPoint: The Best Free DropPoint Alternative for Windows',
  description:
    'Holdem vs DropPoint — Tauri/Rust vs Electron, mouse-shake vs Shift+CapsLock, maintained vs unreleased since 2022. Full free, open-source shelf comparison.',
  keywords: [
    'Holdem vs DropPoint',
    'DropPoint alternative',
    'free DropPoint alternative',
    'DropPoint Windows',
  ],
  alternates: {
    canonical: 'https://holdem.iamzub.in/vs/droppoint',
  },
  openGraph: {
    title:
      'Holdem vs DropPoint: The Best Free DropPoint Alternative for Windows',
    description:
      'Tauri/Rust vs Electron, shake vs Shift+CapsLock, maintained vs stalled. Full Holdem vs DropPoint comparison.',
    url: 'https://holdem.iamzub.in/vs/droppoint',
    images: [
      {
        url: '/og.png',
        width: 1280,
        height: 640,
        alt: 'Holdem vs DropPoint comparison — free file shelf for Windows',
      },
    ],
    type: 'article',
  },
  twitter: {
    card: 'summary_large_image',
    site: '@iamzub_in',
    creator: '@iamzub_in',
    title:
      'Holdem vs DropPoint: The Best Free DropPoint Alternative for Windows',
    description: 'Tauri/Rust vs Electron — the modern open-source upgrade.',
    images: ['/og.png'],
  },
}

const faqs = [
  {
    q: 'Is DropPoint still maintained?',
    a: 'Not actively, as far as its public repo shows: the latest release is v1.2.1 (September 2022) with no newer release since, and the last code push was in 2023. It still works, but bugs and OS changes since then are unaddressed. Holdem is under active development (v3.1.0).',
  },
  {
    q: 'How do Holdem and DropPoint differ technically?',
    a: 'DropPoint is built on Electron + Node.js (installers ~54–78MB per its release assets) while Holdem uses Tauri + Rust, which produces a far smaller binary and near-zero background resource use. Both are free and open-source — DropPoint under GPL-3.0, Holdem under MIT.',
  },
  {
    q: 'How do you summon each shelf?',
    a: 'DropPoint: press Shift+CapsLock anywhere (Shift+Tab on macOS) or click the tray icon — there is no mouse-shake gesture. Holdem: shake the mouse while dragging, or press the customizable global hotkey, and the shelf appears at your cursor.',
  },
  {
    q: 'Does DropPoint work across virtual desktops?',
    a: 'Only with manual setup on Windows: its README instructs you to open Task view, right-click the window, and tick “Show this window on all desktops.” Stickiness works by default on other OSes. Holdem runs as a tray app designed to be reachable everywhere.',
  },
  {
    q: 'What content can each hold?',
    a: 'DropPoint holds files and folders. Holdem holds files, folders, and images dragged directly from the browser, with URL and text-snippet drops on the roadmap.',
  },
  {
    q: 'Which free open-source shelf should I pick?',
    a: 'If you need Windows, macOS, and Linux covered by one tool today, DropPoint is cross-platform and free. If you are on Windows and want the lightest, actively maintained shelf with shake-to-summon and browser-image support, pick Holdem.',
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

export default function VsDroppointPage() {
  return (
    <main className="mx-auto max-w-3xl py-8">
      <script
        type="application/ld+json"
        dangerouslySetInnerHTML={{ __html: JSON.stringify(faqSchema) }}
      />
      <Link href="/" className="text-ink-subtle hover:text-ink text-sm">
        ← Back to Holdem
      </Link>
      <article className="prose prose-invert prose-headings:tracking-tight prose-a:text-primary-hover mt-6 max-w-none">
        <p className="text-ink-subtle text-sm font-medium tracking-widest uppercase">
          Compare • Holdem vs DropPoint
        </p>
        <h1 className="!mb-3 text-3xl font-bold md:text-4xl">
          Holdem vs DropPoint: The Best Free DropPoint Alternative for Windows
        </h1>
        <p className="lead text-ink-muted">
          DropPoint deserves respect: a free, open-source shelf that brought the
          Dropover idea to Windows, macOS, and Linux years ago, earning ~1,000
          GitHub stars. But its last release was v1.2.1 in September 2022, it
          runs on heavyweight Electron, and summoning it means an awkward
          Shift+CapsLock chord. Holdem is the modern, actively maintained answer
          for Windows.
        </p>

        <div className="not-prose border-hairline bg-surface-1 rounded-xl border p-4 text-sm">
          <strong>Verdict:</strong> Need one free shelf across Windows, Mac, and
          Linux right now? DropPoint still does the job. On Windows alone,
          Holdem wins on weight (Tauri/Rust vs Electron), trigger (shake vs
          Shift+CapsLock), maintenance (v3.1.0 vs a 2022 release), and extras
          like browser-image drops.
        </div>

        <h2>Holdem vs DropPoint: side-by-side</h2>
        <div className="not-prose border-hairline overflow-x-auto rounded-xl border">
          <table className="w-full text-left text-sm">
            <thead className="bg-surface-1 text-ink-subtle">
              <tr>
                <th className="px-4 py-3 font-medium">Feature</th>
                <th className="px-4 py-3 font-medium">Holdem</th>
                <th className="px-4 py-3 font-medium">DropPoint</th>
              </tr>
            </thead>
            <tbody className="divide-hairline divide-y">
              <tr>
                <td className="px-4 py-3 font-medium">Pricing &amp; license</td>
                <td className="px-4 py-3">Free, open-source (MIT)</td>
                <td className="px-4 py-3">Free, open-source (GPL-3.0)</td>
              </tr>
              <tr>
                <td className="px-4 py-3 font-medium">Platforms</td>
                <td className="px-4 py-3">Windows 10/11</td>
                <td className="px-4 py-3">Windows, macOS, Linux</td>
              </tr>
              <tr>
                <td className="px-4 py-3 font-medium">Tech stack &amp; size</td>
                <td className="px-4 py-3">
                  Tauri + Rust + TypeScript — tiny installer, near-zero
                  background CPU
                </td>
                <td className="px-4 py-3">
                  Electron + Node.js — release installers run ~54–78MB per asset
                </td>
              </tr>
              <tr>
                <td className="px-4 py-3 font-medium">Trigger</td>
                <td className="px-4 py-3">
                  Mouse shake mid-drag or global hotkey; shelf appears at cursor
                </td>
                <td className="px-4 py-3">
                  Shift+CapsLock anywhere (Shift+Tab on macOS) or tray icon; no
                  shake gesture
                </td>
              </tr>
              <tr>
                <td className="px-4 py-3 font-medium">Supported content</td>
                <td className="px-4 py-3">
                  Files, folders, web images (URLs &amp; text on the roadmap)
                </td>
                <td className="px-4 py-3">Files and folders</td>
              </tr>
              <tr>
                <td className="px-4 py-3 font-medium">Tray &amp; instances</td>
                <td className="px-4 py-3">
                  Quiet tray app, auto-launch, one floating shelf that
                  auto-hides
                </td>
                <td className="px-4 py-3">
                  Minimizes to tray; supports multiple instances; new instances
                  open at mouse location
                </td>
              </tr>
              <tr>
                <td className="px-4 py-3 font-medium">Virtual desktops</td>
                <td className="px-4 py-3">Tray-based, reachable everywhere</td>
                <td className="px-4 py-3">
                  Windows needs a manual Task-view step (“Show this window on
                  all desktops”)
                </td>
              </tr>
              <tr>
                <td className="px-4 py-3 font-medium">Maintenance</td>
                <td className="px-4 py-3">Active — current release v3.1.0</td>
                <td className="px-4 py-3">
                  Stalled — last release v1.2.1 (Sep 2022), last push 2023
                </td>
              </tr>
              <tr>
                <td className="px-4 py-3 font-medium">Design</td>
                <td className="px-4 py-3">Minimal dark floating shelf</td>
                <td className="px-4 py-3">
                  Functional multi-window Electron UI (taste differs — try both)
                </td>
              </tr>
            </tbody>
          </table>
        </div>

        <h2>Weight: why the stack matters in a tray app</h2>
        <p>
          A shelf runs all day in the background, so its runtime is the product.
          DropPoint is an Electron app — the same Chromium + Node foundation as
          Slack and VS Code — and its own release page tells the story: a
          Windows zip around 77MB, a macOS dmg up to ~79MB. That is fine for a
          main app, but heavy for a utility whose job is to hold three files for
          thirty seconds. Holdem is built with Tauri and Rust, which reuses the
          system webview instead of shipping Chromium: the result is a far
          smaller install, negligible idle CPU, and instant summon. Both
          approaches work; only one disappears into the tray.
        </p>

        <h2>Trigger: shake vs Shift+CapsLock</h2>
        <p>
          This is the daily-experience gap. With DropPoint you press{' '}
          <code>Shift + Caps Lock</code> (an unusual chord that collides with
          typing habits) or dig out the tray icon — and on Windows, if you use
          virtual desktops, you must first perform a one-time Task-view ritual
          to pin the window across desktops. With Holdem you simply shake the
          mouse while already dragging — the shelf materializes exactly where
          your cursor is — or tap the hotkey. No chord to memorize, no
          per-desktop setup, nothing between intention and shelf.
        </p>

        <h2>Maintenance: 2022 vs today</h2>
        <p>
          Software rots when operating systems move on. DropPoint&apos;s public
          record — v1.2.1 released September 2022, no release since, last commit
          activity in 2023, 28 open issues — means Windows 11 changes, new
          browsers, and HiDPI quirks land in an app nobody is patching. That is
          not an attack: open-source maintainers move on, and the GPL code
          remains available for anyone to fork. But if you are choosing a shelf
          to rely on daily, an actively developed app (Holdem v3.1.0, Tauri
          auto-updates via <code>update.json</code>) is the safer bet — which is
          exactly why Holdem exists rather than as a DropPoint fork.
        </p>

        <h2>Where DropPoint honestly wins</h2>
        <p>
          Cross-platform. Full stop. If your workflow spans Windows, macOS, and
          Linux, DropPoint is the only free open-source shelf covering all
          three, and its multiple-instances model suits people who keep several
          staging piles open. The GPL-3.0 license also guarantees the code stays
          open, which matters to free-software purists (Holdem&apos;s MIT
          license is more permissive instead). Windows-only users give up
          nothing by switching.
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

        <h2>FAQ: Holdem vs DropPoint</h2>
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
              href="https://github.com/GameGodS3/DropPoint"
              target="_blank"
              rel="noopener noreferrer"
            >
              DropPoint on GitHub
            </a>{' '}
            — GPL-3.0, Electron, Shift+CapsLock / Shift+Tab triggers, tray
            behavior, virtual-desktop step
          </li>
          <li>
            <a
              href="https://github.com/GameGodS3/DropPoint/releases/tag/v1.2.0"
              target="_blank"
              rel="noopener noreferrer"
            >
              DropPoint v1.2.0 release assets
            </a>{' '}
            — installer sizes (~54–79MB across platforms)
          </li>
          <li>
            <a
              href="https://droppoint.netlify.app/"
              target="_blank"
              rel="noopener noreferrer"
            >
              DropPoint homepage
            </a>{' '}
            — feature rundown, hotkey, tray
          </li>
        </ul>

        <hr />
        <p className="text-ink-subtle text-sm">
          Also: <Link href="/vs/dropshelf">Holdem vs Dropshelf</Link> •{' '}
          <Link href="/vs/dropover">Holdem vs Dropover</Link> •{' '}
          <Link href="/vs/yoink">Holdem vs Yoink</Link> •{' '}
          <Link href="/">Go to homepage →</Link>
        </p>
      </article>
    </main>
  )
}
