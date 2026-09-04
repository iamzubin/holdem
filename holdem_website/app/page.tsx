'use client'
import { motion } from 'motion/react'
import Link from 'next/link'
import { HoldemDemo } from './components/holdem-demo'
import { Button, ButtonLink } from './components/ui/button'
import { useEffect, useState } from 'react'
import { DOWNLOAD_URL_FALLBACK } from '@/lib/constants'

// Below-fold sections animate on scroll into view — hero stays plain HTML
// so crawlers and LCP never wait on opacity:0/blur JS.
const VARIANTS_SECTION = {
  hidden: { opacity: 0, y: 20 },
  visible: { opacity: 1, y: 0 },
}

const TRANSITION_SECTION = {
  duration: 0.3,
}

const FAQS = [
  {
    q: 'Is Holdem really free?',
    a: 'Yes. Holdem is 100% free and open-source (MIT). No wait timers, no pro unlocks, no paywall — unlike Dropshelf free tier with its 3-second wait.',
  },
  {
    q: 'Which Windows versions are supported?',
    a: 'Windows 10 and Windows 11 (x64). Download the .exe installer from the download page or GitHub Releases.',
  },
  {
    q: 'How do I summon the shelf?',
    a: 'Shake your mouse while dragging files, or press the global hotkey. The floating shelf appears at your cursor — drop files in, navigate anywhere, then drag them out.',
  },
  {
    q: 'What can I drop on the shelf?',
    a: 'Files, folders, and images dragged directly from the browser. URL and text-snippet support is on the roadmap — see the changelog.',
  },
  {
    q: 'Is Holdem a Dropover alternative for Windows?',
    a: 'Yes. Dropover is Mac-only. Holdem brings the same temporary-shelf workflow to Windows, free and open-source, built with Tauri/Rust for low memory use.',
  },
]

const faqSchema = {
  '@context': 'https://schema.org',
  '@type': 'FAQPage',
  mainEntity: FAQS.map((f) => ({
    '@type': 'Question',
    name: f.q,
    acceptedAnswer: { '@type': 'Answer', text: f.a },
  })),
}

const Feature = ({
  title,
  description,
}: {
  title: string
  description: string
}) => (
  <div className="mb-4">
    <p className="flex items-center gap-2 text-lg font-medium">
      <span className="text-green-500" aria-hidden="true">
        ✅
      </span>{' '}
      {title}
    </p>
    <p className="pl-7 text-zinc-600 dark:text-zinc-400">{description}</p>
  </div>
)

const ComingSoonFeature = ({
  title,
  description,
}: {
  title: string
  description: string
}) => (
  <div className="mb-4">
    <p className="flex items-center gap-2 text-lg font-medium">
      <span className="text-yellow-500" aria-hidden="true">
        🚧
      </span>{' '}
      {title}
    </p>
    <p className="pl-7 text-zinc-600 dark:text-zinc-400">{description}</p>
  </div>
)

function DemoSwitcher() {
  // PC (fine pointer): interactive demo. Phone/tab (coarse pointer): video.
  // Both stay in the DOM so crawlers always see the video element.
  const [showVideo, setShowVideo] = useState(false)

  useEffect(() => {
    if (window.matchMedia('(pointer: coarse)').matches) {
      setShowVideo(true)
    }
  }, [])

  return (
    <div className="mx-auto mt-12 w-full max-w-full">
      <div className={showVideo ? 'hidden' : 'block'}>
        <HoldemDemo />
      </div>
      <figure className={`mx-auto max-w-3xl ${showVideo ? 'block' : 'hidden'}`}>
        <video
          controls
          preload="none"
          poster="/og.png"
          className="w-full rounded-2xl border border-zinc-200 shadow-lg dark:border-zinc-800"
          aria-label="Holdem demo video: shake mouse while dragging to summon the floating file shelf on Windows"
        >
          <source src="/assets/herovideo.mp4" type="video/mp4" />
          Your browser does not support the video tag.
        </video>
        <figcaption className="mt-2 text-sm text-zinc-500">
          Watch: shake to summon the shelf, stash files, drop anywhere on
          Windows.
        </figcaption>
      </figure>
      <div className="mt-1.5 text-center">
        <button
          onClick={() => setShowVideo((v) => !v)}
          aria-pressed={showVideo}
          className="text-[11px] text-zinc-400 underline-offset-2 hover:text-zinc-600 hover:underline dark:hover:text-zinc-300"
        >
          {showVideo ? '✦ try the interactive demo' : '▶ watch the video'}
        </button>
      </div>
    </div>
  )
}

function ShareModal({ open, onClose }: { open: boolean; onClose: () => void }) {
  const [copied, setCopied] = useState(false)
  const url = 'https://holdem.iamzub.in/'
  const text = 'Holdem — Free drag-and-drop file shelf for Windows'
  const encodedUrl = encodeURIComponent(url)
  const encodedText = encodeURIComponent(text)

  const links = [
    {
      label: 'X / Twitter',
      href: `https://twitter.com/intent/tweet?text=${encodedText}&url=${encodedUrl}`,
      icon: 'X',
    },
    {
      label: 'WhatsApp',
      href: `https://wa.me/?text=${encodedText}%20${encodedUrl}`,
      icon: 'WA',
    },
    {
      label: 'LinkedIn',
      href: `https://www.linkedin.com/sharing/share-offsite/?url=${encodedUrl}`,
      icon: 'in',
    },
    {
      label: 'Facebook',
      href: `https://www.facebook.com/sharer/sharer.php?u=${encodedUrl}`,
      icon: 'Fb',
    },
    {
      label: 'Reddit',
      href: `https://www.reddit.com/submit?url=${encodedUrl}&title=${encodedText}`,
      icon: 'Rd',
    },
    {
      label: 'Email',
      href: `mailto:?subject=${encodedText}&body=${encodedText}%0A${encodedUrl}`,
      icon: '@',
    },
  ]

  useEffect(() => {
    if (!open) return
    const onKey = (e: KeyboardEvent) => e.key === 'Escape' && onClose()
    document.addEventListener('keydown', onKey)
    document.body.style.overflow = 'hidden'
    return () => {
      document.removeEventListener('keydown', onKey)
      document.body.style.overflow = ''
    }
  }, [open, onClose])

  if (!open) return null

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
      <div
        className="absolute inset-0 bg-black/40 backdrop-blur-sm"
        onClick={onClose}
      />
      <motion.div
        initial={{ opacity: 0, y: 10, scale: 0.98 }}
        animate={{ opacity: 1, y: 0, scale: 1 }}
        exit={{ opacity: 0, y: 10 }}
        className="relative w-full max-w-md rounded-2xl border border-zinc-200 bg-white p-6 shadow-xl dark:border-zinc-800 dark:bg-zinc-900"
        role="dialog"
        aria-modal="true"
      >
        <button
          onClick={onClose}
          aria-label="Close"
          className="absolute top-4 right-4 rounded-full p-1 text-zinc-500 hover:bg-zinc-100 dark:hover:bg-zinc-800"
        >
          ✕
        </button>
        <h2 className="text-lg font-semibold">Share Holdem with friends</h2>
        <p className="mt-1 text-sm text-zinc-600 dark:text-zinc-400">
          If you like Holdem, help others discover it.
        </p>

        <div className="mt-5 grid grid-cols-3 gap-2">
          {links.map((l) => (
            <a
              key={l.label}
              href={l.href}
              target="_blank"
              rel="noopener noreferrer"
              className="flex flex-col items-center justify-center gap-1 rounded-xl border border-zinc-200 bg-zinc-50 px-2 py-3 text-sm font-medium hover:bg-zinc-100 dark:border-zinc-800 dark:bg-zinc-800 dark:hover:bg-zinc-700"
            >
              <span className="text-base">{l.icon}</span>
              {l.label}
            </a>
          ))}
        </div>

        <div className="mt-4 flex items-center gap-2">
          <input
            readOnly
            value={url}
            aria-label="Share URL"
            className="flex-1 rounded-lg border border-zinc-200 bg-zinc-50 px-3 py-2 text-sm dark:border-zinc-800 dark:bg-zinc-800"
          />
          <Button
            variant="primary"
            size="sm"
            onClick={async () => {
              await navigator.clipboard.writeText(url)
              setCopied(true)
              setTimeout(() => setCopied(false), 1500)
            }}
          >
            {copied ? 'Copied!' : 'Copy'}
          </Button>
        </div>

        <div className="my-5 h-px bg-zinc-100 dark:bg-zinc-800" />

        <a
          href="https://www.buymeacoffee.com/iamzubin"
          target="_blank"
          rel="noopener noreferrer"
          className="flex w-full items-center justify-center gap-2 rounded-lg bg-[#FFDD00] px-4 py-2.5 text-sm font-semibold text-zinc-900 hover:bg-[#f5d500]"
        >
          <span>🍟</span> Buy me some fries
        </a>
        <p className="mt-2 text-center text-xs text-zinc-500">
          Support the solo dev behind Holdem
        </p>
      </motion.div>
    </div>
  )
}

export default function Home() {
  // Never empty: seed with the known-good GitHub release URL so the CTA
  // works with JS disabled, before fetch resolves, or if update.json fails.
  const [downloadUrl, setDownloadUrl] = useState<string>(DOWNLOAD_URL_FALLBACK)
  const [downloads, setDownloads] = useState<number | null>(null)
  const [shareOpen, setShareOpen] = useState(false)

  useEffect(() => {
    fetch('/update.json')
      .then((res) => res.json())
      .then((data) => {
        const windowsUrl = data.platforms?.['windows-x86_64']?.url
        if (typeof windowsUrl === 'string' && windowsUrl.startsWith('http')) {
          setDownloadUrl(windowsUrl)
        }
      })
      .catch(() => {
        // keep DOWNLOAD_URL_FALLBACK
      })

    fetch('https://api.github.com/repos/iamzubin/holdem/releases')
      .then((res) => res.json())
      .then((data) => {
        if (Array.isArray(data)) {
          const totalDownloads = data.reduce((acc: number, release: any) => {
            const assetDownloads =
              release.assets?.reduce(
                (a: number, asset: any) => a + (asset.download_count || 0),
                0,
              ) || 0
            return acc + assetDownloads
          }, 0)
          setDownloads(totalDownloads)
        }
      })
      .catch(() => setDownloads(null))
  }, [])

  return (
    <main className="space-y-20">
      <script
        type="application/ld+json"
        dangerouslySetInnerHTML={{ __html: JSON.stringify(faqSchema) }}
      />
      {/* Hero Section — plain SSR HTML, no opacity:0/blur gate for LCP & crawlers */}
      <section className="py-12 text-center">
        <p className="mb-4 text-4xl font-bold">Holdem</p>
        <h1 className="mx-auto mb-6 max-w-3xl text-3xl font-bold tracking-tight text-zinc-900 md:text-4xl dark:text-zinc-50">
          Holdem — the free Dropover for Windows
        </h1>
        <p className="mx-auto mb-8 max-w-2xl text-zinc-600 dark:text-zinc-400">
          Holdem is a lightweight, free and open-source shelf for Windows that
          makes moving files effortless. Shake your mouse while dragging to
          summon a floating holding area, collect files, images, and folders
          from anywhere — then drop them exactly where you need them. Built with
          Tauri for speed and privacy, it runs quietly in your system tray and
          keeps your desktop clutter-free.
        </p>

        <div className="flex flex-col items-center justify-center gap-3">
          <ButtonLink
            variant="primary"
            size="md"
            href={downloadUrl}
            download
            target="_blank"
            rel="noopener noreferrer"
          >
            ⬇️ Download for Windows — Free
            {downloads !== null && (
              <span className="ml-1 text-xs opacity-70">
                ({downloads.toLocaleString()} downloads)
              </span>
            )}
          </ButtonLink>
          <p className="text-xs text-zinc-500">
            Windows 10/11 x64 · v3.1.0 · Free &amp; open-source ·{' '}
            <a
              href="https://github.com/iamzubin/holdem/releases"
              target="_blank"
              rel="noopener noreferrer"
              className="underline hover:text-zinc-800 dark:hover:text-zinc-200"
            >
              All releases
            </a>
          </p>
          <Button
            variant="secondary"
            size="sm"
            onClick={() => setShareOpen(true)}
          >
            <span className="text-base">↗</span> Share with friends
          </Button>
        </div>
        <ShareModal open={shareOpen} onClose={() => setShareOpen(false)} />

        <DemoSwitcher />
      </section>

      {/* How it works */}
      <motion.section
        variants={VARIANTS_SECTION}
        initial="hidden"
        whileInView="visible"
        viewport={{ once: true, margin: '-80px' }}
        transition={TRANSITION_SECTION}
      >
        <h2 className="mb-6 text-2xl font-bold">How it works</h2>
        <h3 className="mb-4 text-xl font-medium">Shake. Drop. Done.</h3>

        <p className="mb-8 text-zinc-600 dark:text-zinc-400">
          Holdem runs quietly in your system tray. When you need to move files,
          simply shake your mouse while dragging — a floating shelf appears,
          ready to hold your files temporarily. Drag them out when you&apos;re
          ready. See the{' '}
          <Link href="/docs" className="underline">
            docs
          </Link>{' '}
          for hotkeys and tips.
        </p>

        <h3 className="mb-4 text-xl font-medium">
          Floating Shelf for Your Files
        </h3>

        <p className="mb-8 text-zinc-600 dark:text-zinc-400">
          Keep your files within reach while switching between folders, apps, or
          desktops. Holdem helps you multitask without losing track of what
          you&apos;re moving.
        </p>
      </motion.section>

      {/* Features */}
      <motion.section
        variants={VARIANTS_SECTION}
        initial="hidden"
        whileInView="visible"
        viewport={{ once: true, margin: '-80px' }}
        transition={TRANSITION_SECTION}
      >
        <h2 className="mb-6 text-2xl font-bold">Features</h2>

        <div className="grid gap-4 md:grid-cols-2">
          <div>
            <Feature
              title="Floating Holding Area"
              description="Instantly accessible, draggable shelf to drop and hold files."
            />
            <Feature
              title="Drag and Drop Support"
              description="Drag files or folders in and out of the shelf with ease."
            />
            <Feature
              title="Browser Image Support"
              description="Drop images directly from your web browser into the shelf."
            />
          </div>
          <div>
            <Feature
              title="System Tray Integration"
              description="Access Holdem with a right-click from your tray."
            />
            <Feature
              title="Global Hotkey"
              description="Bring up the shelf with a custom key combo."
            />
            <Feature
              title="Auto-launch on Startup"
              description="Always ready when you log in."
            />
          </div>
        </div>
      </motion.section>

      {/* Coming soon */}
      <motion.section
        variants={VARIANTS_SECTION}
        initial="hidden"
        whileInView="visible"
        viewport={{ once: true, margin: '-80px' }}
        transition={TRANSITION_SECTION}
        className="mb-12"
      >
        <h2 className="mb-6 text-2xl font-bold">Coming Soon</h2>

        <div className="grid gap-4 md:grid-cols-2">
          <ComingSoonFeature
            title="Drop anything"
            description="Support for images, text, links — not just files."
          />
          <ComingSoonFeature
            title="Multiple Shelves"
            description="Organize files into separate shelves for different tasks."
          />
        </div>
      </motion.section>

      {/* FAQ — crawlable, keyword-rich, powers FAQ schema */}
      <motion.section
        variants={VARIANTS_SECTION}
        initial="hidden"
        whileInView="visible"
        viewport={{ once: true, margin: '-80px' }}
        transition={TRANSITION_SECTION}
      >
        <h2 className="mb-6 text-2xl font-bold">Frequently asked questions</h2>
        <div className="space-y-4">
          {FAQS.map((f) => (
            <div
              key={f.q}
              className="rounded-xl border border-zinc-200 p-4 dark:border-zinc-800"
            >
              <h3 className="font-medium">{f.q}</h3>
              <p className="mt-1 text-sm text-zinc-600 dark:text-zinc-400">
                {f.a}
              </p>
            </div>
          ))}
        </div>
        <div className="mt-6 flex flex-wrap gap-3 text-sm">
          <Link
            href="/pricing"
            className="text-zinc-600 underline hover:text-zinc-900 dark:text-zinc-400"
          >
            Pricing — free forever
          </Link>
          <Link
            href="/changelog"
            className="text-zinc-600 underline hover:text-zinc-900 dark:text-zinc-400"
          >
            Changelog
          </Link>
          <Link
            href="/docs"
            className="text-zinc-600 underline hover:text-zinc-900 dark:text-zinc-400"
          >
            Docs
          </Link>
          <Link
            href="/vs/dropover"
            className="text-zinc-600 underline hover:text-zinc-900 dark:text-zinc-400"
          >
            Holdem vs Dropover
          </Link>
        </div>
      </motion.section>
    </main>
  )
}
