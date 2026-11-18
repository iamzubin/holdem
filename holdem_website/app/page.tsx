'use client'
import { useEffect, useState } from 'react'
import { AnimatePresence, motion, MotionConfig } from 'motion/react'
import Link from 'next/link'
import { Coffee, Download, Share2, X } from 'lucide-react'
import { HoldemDemo } from './components/holdem-demo'
import { Button, ButtonLink } from './components/ui/button'
import { cn } from '@/lib/utils'
import { StoreBadge } from './components/store-badge'
import {
  DOWNLOAD_URL_FALLBACK,
  STORE_PRICE,
  STORE_TRIAL_DAYS,
} from '@/lib/constants'

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
    a: 'Yes. The installer on GitHub is free and open source (MIT), with no wait timers and nothing locked. Holdem is also on the Microsoft Store for $4.99 with a 7-day free trial. It is the same app; buying it there supports development and gets you updates through the Store.',
  },
  {
    q: 'What is the difference between the free download and the Microsoft Store version?',
    a: 'Features are identical. The Store version installs and updates through the Microsoft Store, works on every PC signed in to your Microsoft account, and pays for continued development. The free .exe from GitHub updates itself.',
  },
  {
    q: 'Which Windows versions are supported?',
    a: 'Windows 10 version 1809 or later and Windows 11, 64-bit (x64). Get it from the Microsoft Store, or download the .exe installer from the download page or GitHub Releases.',
  },
  {
    q: 'How do I summon the shelf?',
    a: 'Shake your mouse while dragging files, or press the global hotkey (Win+Shift+H by default). You can also turn on opening the shelf by holding a drag against a screen edge. Drop files in, go anywhere, then drag them out.',
  },
  {
    q: 'What can I drop on the shelf?',
    a: 'Files and folders, images and links from the browser, text snippets, email attachments, emails from classic Outlook, images from chat apps, and files from ZIP folders, 7-Zip or WinRAR. You can also paste onto the shelf with Ctrl+V.',
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
  <div>
    <p className="font-medium">{title}</p>
    <p className="text-ink-muted mt-1 text-sm">{description}</p>
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
      <div className={cn(!showVideo && 'block', showVideo && 'hidden')}>
        <HoldemDemo />
      </div>
      <figure
        className={cn(
          'mx-auto max-w-3xl',
          showVideo && 'block',
          !showVideo && 'hidden',
        )}
      >
        <video
          controls
          preload="none"
          muted
          playsInline
          poster="/assets/hero-poster.jpg"
          className="border-hairline bg-surface-1 w-full rounded-2xl border shadow-lg"
          aria-label="Holdem demo video: the shelf peeks in from the screen edge during a drag, collects files from Explorer, Teams and Edge, and drags them all into an Outlook email"
        >
          <source src="/assets/herovideo.mp4" type="video/mp4" />
          Your browser does not support the video tag.
        </video>
        <figcaption className="text-ink-subtle mt-2 text-sm">
          Collect files from several places onto the shelf, then drag them all
          out at once.
        </figcaption>
      </figure>
      <div className="mt-1.5 text-center">
        <button
          onClick={() => setShowVideo((v) => !v)}
          aria-pressed={showVideo}
          className="text-ink-subtle hover:text-ink text-[11px] underline-offset-2 hover:underline"
        >
          {showVideo ? 'Try the interactive demo' : 'Watch the video'}
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
        className="border-hairline bg-surface-1 relative w-full max-w-md rounded-2xl border p-6 shadow-xl"
        role="dialog"
        aria-modal="true"
      >
        <button
          onClick={onClose}
          aria-label="Close"
          className="text-ink-subtle hover:bg-surface-2 absolute top-4 right-4 rounded-full p-1"
        >
          <X className="h-4 w-4" />
        </button>
        <h2 className="text-lg font-semibold">Share Holdem with friends</h2>
        <p className="text-ink-muted mt-1 text-sm">
          If you like Holdem, help others discover it.
        </p>

        <div className="mt-5 grid grid-cols-3 gap-2">
          {links.map((l) => (
            <a
              key={l.label}
              href={l.href}
              target="_blank"
              rel="noopener noreferrer"
              className="border-hairline bg-surface-2 hover:bg-surface-3 flex flex-col items-center justify-center gap-1 rounded-xl border px-2 py-3 text-sm font-medium"
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
            className="border-hairline bg-surface-2 flex-1 rounded-md border px-3 py-2 text-sm"
          />
          <Button
            variant="primary"
            size="sm"
            onClick={async () => {
              try {
                await navigator.clipboard.writeText(url)
                setCopied(true)
                setTimeout(() => setCopied(false), 1500)
              } catch (error) {
                console.error('Failed to copy Holdem share URL:', error)
              }
            }}
          >
            {copied ? 'Copied!' : 'Copy'}
          </Button>
        </div>

        <div className="bg-hairline my-5 h-px" />

        <a
          href="https://www.buymeacoffee.com/iamzubin"
          target="_blank"
          rel="noopener noreferrer"
          className="border-hairline bg-surface-2 text-ink hover:bg-surface-3 flex w-full items-center justify-center gap-2 rounded-md border px-4 py-2.5 text-sm font-semibold"
        >
          <Coffee className="h-4 w-4" /> Buy me some fries
        </a>
        <p className="text-ink-subtle mt-2 text-center text-xs">
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
    <MotionConfig reducedMotion="user">
      <main className="space-y-20">
        <script
          type="application/ld+json"
          dangerouslySetInnerHTML={{ __html: JSON.stringify(faqSchema) }}
        />
        {/* Hero Section — plain SSR HTML, no opacity:0/blur gate for LCP & crawlers */}
        <section className="py-12 text-center">
          <p className="mb-4 text-4xl font-bold">Holdem</p>
          <h1 className="text-ink mx-auto mb-6 max-w-3xl text-3xl font-bold tracking-[-0.02em] md:text-4xl">
            Holdem — the free Dropover for Windows
          </h1>
          <p className="text-ink-muted mx-auto mb-8 max-w-2xl">
            Holdem is a lightweight, open-source shelf for Windows that makes
            moving files effortless. Shake your mouse while dragging to summon a
            floating holding area, collect files, images, links and text from
            anywhere, then drop them exactly where you need them. Built with
            Tauri for speed and privacy, it runs quietly in your system tray and
            never sends your files anywhere.
          </p>

          <div className="flex flex-col items-center justify-center gap-3">
            <StoreBadge />
            <p className="text-ink-subtle text-xs">
              {STORE_PRICE} on the Microsoft Store · {STORE_TRIAL_DAYS}-day free
              trial · supports development
            </p>
            <ButtonLink
              variant="secondary"
              size="md"
              href={downloadUrl}
              download
              target="_blank"
              rel="noopener noreferrer"
            >
              <Download className="h-4 w-4" /> Or download free from GitHub
              {downloads !== null && (
                <span className="ml-1 text-xs opacity-70">
                  ({downloads.toLocaleString()} downloads)
                </span>
              )}
            </ButtonLink>
            <p className="text-ink-subtle text-xs">
              Windows 10/11 x64 · v3.2.0 · Same app, free &amp; open-source
              (MIT) ·{' '}
              <a
                href="https://github.com/iamzubin/holdem/releases"
                target="_blank"
                rel="noopener noreferrer"
                className="hover:text-ink underline"
              >
                All releases
              </a>
            </p>
            <Button
              variant="secondary"
              size="sm"
              onClick={() => setShareOpen(true)}
            >
              <Share2 className="h-4 w-4" /> Share with friends
            </Button>
          </div>
          <AnimatePresence>
            {shareOpen && (
              <ShareModal
                open={shareOpen}
                onClose={() => setShareOpen(false)}
              />
            )}
          </AnimatePresence>

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

          <p className="text-ink-muted mb-8">
            Holdem runs quietly in your system tray. When you need to move
            files, simply shake your mouse while dragging — a floating shelf
            appears, ready to hold your files temporarily. Drag them out when
            you&apos;re ready. See the{' '}
            <Link href="/docs" className="underline">
              docs
            </Link>{' '}
            for hotkeys and tips.
          </p>

          <h3 className="mb-4 text-xl font-medium">
            Floating Shelf for Your Files
          </h3>

          <p className="text-ink-muted mb-8">
            Keep your files within reach while switching between folders, apps,
            or desktops. Holdem helps you multitask without losing track of what
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

          <div className="grid gap-6 md:grid-cols-2">
            <Feature
              title="Floating Holding Area"
              description="Instantly accessible, draggable shelf to drop and hold files."
            />
            <Feature
              title="Drag and Drop Support"
              description="Drag files or folders in and out of the shelf with ease."
            />
            <Feature
              title="Drop Almost Anything"
              description="Browser images and links, text snippets, email attachments, Outlook emails, chat images, and files from ZIP, 7-Zip or WinRAR."
            />
            <Feature
              title="Multiple Shelves"
              description="Keep a shelf per task as tabs. Reopen a closed shelf with Ctrl+Shift+T for up to a week."
            />
            <Feature
              title="Preview and Item Actions"
              description="Preview with Space, copy as files with Ctrl+C, copy to a folder, zip, pin, and undo."
            />
            <Feature
              title="Image Resize and Convert"
              description="Resize or convert to PNG or JPEG. The original is never changed."
            />
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
              description="Always ready when you log in. Your shelves survive restarts and updates."
            />
          </div>
        </motion.section>

        {/* Two ways to get it */}
        <motion.section
          variants={VARIANTS_SECTION}
          initial="hidden"
          whileInView="visible"
          viewport={{ once: true, margin: '-80px' }}
          transition={TRANSITION_SECTION}
          className="mb-12"
        >
          <h2 className="mb-6 text-2xl font-bold">Two ways to get Holdem</h2>

          <div className="grid gap-4 md:grid-cols-2">
            <div className="border-hairline bg-surface-1 flex flex-col rounded-2xl border p-5">
              <p className="font-medium">Microsoft Store · {STORE_PRICE}</p>
              <p className="text-ink-muted mt-1 mb-4 flex-1 text-sm">
                Try it free for {STORE_TRIAL_DAYS} days. Installs and updates
                through the Store, works on every PC signed in to your Microsoft
                account, and pays for new features.
              </p>
              <StoreBadge height={40} />
            </div>
            <div className="border-hairline bg-surface-1 flex flex-col rounded-2xl border p-5">
              <p className="font-medium">GitHub · Free</p>
              <p className="text-ink-muted mt-1 mb-4 flex-1 text-sm">
                The same app, MIT-licensed. Download the installer and it
                updates itself. Nothing is locked and there are no timers.
              </p>
              <ButtonLink
                variant="secondary"
                size="md"
                href={downloadUrl}
                download
                className="self-start"
              >
                <Download className="h-4 w-4" /> Download .exe
              </ButtonLink>
            </div>
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
          <h2 className="mb-6 text-2xl font-bold">
            Frequently asked questions
          </h2>
          <div className="space-y-4">
            {FAQS.map((f) => (
              <div
                key={f.q}
                className="border-hairline bg-surface-1 rounded-xl border p-4"
              >
                <h3 className="font-medium">{f.q}</h3>
                <p className="text-ink-muted mt-1 text-sm">{f.a}</p>
              </div>
            ))}
          </div>
          <div className="mt-6 flex flex-wrap gap-3 text-sm">
            <Link
              href="/pricing"
              className="text-ink-muted hover:text-ink underline"
            >
              Pricing
            </Link>
            <Link
              href="/changelog"
              className="text-ink-muted hover:text-ink underline"
            >
              Changelog
            </Link>
            <Link
              href="/docs"
              className="text-ink-muted hover:text-ink underline"
            >
              Docs
            </Link>
            <Link
              href="/vs/dropover"
              className="text-ink-muted hover:text-ink underline"
            >
              Holdem vs Dropover
            </Link>
          </div>
        </motion.section>
      </main>
    </MotionConfig>
  )
}
