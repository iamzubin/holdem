'use client'
import { motion } from 'motion/react'
import Link from 'next/link'
import Image from 'next/image'
import { HoldemDemo } from './components/holdem-demo'
import { useEffect, useState } from 'react'

const VARIANTS_CONTAINER = {
  hidden: { opacity: 0 },
  visible: {
    opacity: 1,
    transition: {
      staggerChildren: 0.15,
    },
  },
}

const VARIANTS_SECTION = {
  hidden: { opacity: 0, y: 20, filter: 'blur(8px)' },
  visible: { opacity: 1, y: 0, filter: 'blur(0px)' },
}

const TRANSITION_SECTION = {
  duration: 0.3,
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
      <span className="text-green-500">✅</span> {title}
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
      <span className="text-yellow-500">🚧</span> {title}
    </p>
    <p className="pl-7 text-zinc-600 dark:text-zinc-400">{description}</p>
  </div>
)

function GitHubStarButton() {
  const [stars, setStars] = useState<number | null>(null)

  useEffect(() => {
    fetch('https://api.github.com/repos/iamzubin/holdem')
      .then((res) => res.json())
      .then((data) => setStars(data.stargazers_count))
      .catch(() => setStars(null))
  }, [])

  return (
    <a
      href="https://github.com/iamzubin/holdem"
      target="_blank"
      rel="noopener noreferrer"
      className="flex items-center gap-2 rounded-lg border border-transparent bg-zinc-100 px-5 py-2 font-medium text-zinc-900 shadow transition-colors hover:bg-zinc-200 dark:bg-zinc-800 dark:text-zinc-100 dark:hover:bg-zinc-700"
    >
      <svg
        width="18"
        height="18"
        viewBox="0 0 24 24"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
      >
        <path
          d="M12 2L14.7553 8.51147L21.9021 9.23607L16.4511 13.9885L18.1803 21.0139L12 17.5L5.81966 21.0139L7.54894 13.9885L2.09789 9.23607L9.24472 8.51147L12 2Z"
          fill="#FFD600"
          stroke="#FFD600"
          strokeWidth="1.5"
          strokeLinejoin="round"
        />
      </svg>
      <span className="font-bold">Star on Github</span>
      <span className="ml-1 text-base font-semibold">
        {stars !== null ? stars : '—'}
      </span>
    </a>
  )
}

function ShareModal({ open, onClose }: { open: boolean; onClose: () => void }) {
  const [copied, setCopied] = useState(false)
  const url = 'https://holdem.iamzub.in/'
  const text = 'Holdem — A simpler way to manage your files'
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
      <div className="absolute inset-0 bg-black/40 backdrop-blur-sm" onClick={onClose} />
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
            className="flex-1 rounded-lg border border-zinc-200 bg-zinc-50 px-3 py-2 text-sm dark:border-zinc-800 dark:bg-zinc-800"
          />
          <button
            onClick={async () => {
              await navigator.clipboard.writeText(url)
              setCopied(true)
              setTimeout(() => setCopied(false), 1500)
            }}
            className="rounded-lg bg-zinc-900 px-4 py-2 text-sm font-medium text-white hover:bg-zinc-800 dark:bg-white dark:text-zinc-900"
          >
            {copied ? 'Copied!' : 'Copy'}
          </button>
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
        <p className="mt-2 text-center text-xs text-zinc-500">Support the solo dev behind Holdem</p>
      </motion.div>
    </div>
  )
}

export default function Home() {
  const [downloadUrl, setDownloadUrl] = useState<string>('')
  const [downloads, setDownloads] = useState<number | null>(null)
  const [shareOpen, setShareOpen] = useState(false)

  useEffect(() => {
    fetch('/update.json')
      .then((res) => res.json())
      .then((data) => {
        const windowsUrl = data.platforms['windows-x86_64']?.url
        if (windowsUrl) {
          setDownloadUrl(windowsUrl)
        }
      })
      .catch(() => {
        // Fallback to hardcoded URL if fetch fails
        setDownloadUrl(
          'https://github.com/iamzubin/holdem/releases/download/0.2.0/holdem_0.2.0_x64-setup.exe',
        )
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
    <motion.main
      className="space-y-20"
      variants={VARIANTS_CONTAINER}
      initial="hidden"
      animate="visible"
    >
      {/* Hero Section */}
      <motion.section
        className="py-12 text-center"
        variants={VARIANTS_SECTION}
        transition={TRANSITION_SECTION}
      >
        <div className="mb-4 text-4xl font-bold">Holdem</div>
        <h1 className="mb-6 text-xl font-normal text-zinc-600 dark:text-zinc-400">
          A simpler way to manage your files
        </h1>
        <p className="mx-auto mb-8 max-w-2xl text-zinc-600 dark:text-zinc-400">
          Holdem is a lightweight, open-source shelf for Windows that makes
          moving files effortless. Shake your mouse while dragging to summon a
          floating holding area, collect files, images, and folders from anywhere
          — then drop them exactly where you need them. Built with Tauri for
          speed and privacy, it runs quietly in your system tray and keeps your
          desktop clutter-free.
        </p>

        <div className="flex flex-col items-center justify-center gap-3">
          <a
            href={downloadUrl}
            className="flex items-center gap-2 rounded-lg border border-transparent bg-zinc-900 px-5 py-2 font-medium text-white shadow transition-colors hover:bg-zinc-800 dark:border-zinc-200 dark:bg-white dark:text-zinc-900 dark:hover:bg-zinc-100"
            target="_blank"
            rel="noopener noreferrer"
          >
            ⬇️ Download for Windows
            {downloads !== null && (
              <span className="ml-1 text-xs opacity-70">
                ({downloads.toLocaleString()} downloads)
              </span>
            )}
          </a>
          <button
            onClick={() => setShareOpen(true)}
            className="inline-flex items-center gap-2 rounded-lg border border-zinc-200 bg-white px-4 py-1.5 text-sm font-medium text-zinc-900 shadow-sm hover:bg-zinc-50 dark:border-zinc-700 dark:bg-zinc-900 dark:text-zinc-100 dark:hover:bg-zinc-800"
          >
            <span className="text-base">↗</span> Share with friends
          </button>
        </div>
        <ShareModal open={shareOpen} onClose={() => setShareOpen(false)} />

        <div className="mx-auto mt-12 w-full max-w-full">
          <HoldemDemo />
        </div>
      </motion.section>

      {/* How it works */}
      <motion.section
        variants={VARIANTS_SECTION}
        transition={TRANSITION_SECTION}
      >
        <h2 className="mb-6 text-2xl font-bold">How it works</h2>
        <h3 className="mb-4 text-xl font-medium">Shake. Drop. Done.</h3>

        <p className="mb-8 text-zinc-600 dark:text-zinc-400">
          Holdem runs quietly in your system tray. When you need to move files,
          simply shake your mouse while dragging — a floating shelf appears,
          ready to hold your files temporarily. Drag them out when you're ready.
        </p>

        <h3 className="mb-4 text-xl font-medium">
          Floating Shelf for Your Files
        </h3>

        <p className="mb-8 text-zinc-600 dark:text-zinc-400">
          Keep your files within reach while switching between folders, apps, or
          desktops. Holdem helps you multitask without losing track of what
          you're moving.
        </p>
      </motion.section>

      {/* Features */}
      <motion.section
        variants={VARIANTS_SECTION}
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
    </motion.main>
  )
}
