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

export default function Home() {
  const [downloadUrl, setDownloadUrl] = useState<string>('')
  const [downloads, setDownloads] = useState<number | null>(null)

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
        <h1 className="mb-4 text-4xl font-bold">Holdem</h1>
        <p className="mb-6 text-xl text-zinc-600 dark:text-zinc-400">
          A simpler way to manage your files.
        </p>
        <p className="mx-auto mb-8 max-w-2xl text-zinc-600 dark:text-zinc-400">
          Holdem is a lightweight, open-source tool for effortlessly dragging
          and organizing files on your desktop. Inspired by Dropover and built
          with Tauri, Holdem lets you collect and drop files in a temporary
          holding area — no clutter, no hassle.
        </p>

        <div className="flex flex-row items-center justify-center gap-3">
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
          <a
            href="https://www.buymeacoffee.com/iamzubin"
            target="_blank"
            rel="noopener noreferrer"
            className="inline-flex items-center rounded-lg border-none bg-transparent px-0 py-0 font-semibold shadow-none hover:bg-transparent"
            style={{ height: '40px' }}
          >
            <img
              src="https://img.buymeacoffee.com/button-api/?text=Buy me some fries&emoji=🍟&slug=iamzubin&button_colour=353535&font_colour=ffffff&font_family=Lato&outline_colour=ffffff&coffee_colour=FFDD00"
              alt="Buy me some fries"
              style={{ height: '40px', borderRadius: '8px' }}
            />
          </a>
        </div>
        <div className="mt-2 text-center">
          <a
            href="https://github.com/iamzubin/holdem"
            className="text-sm text-zinc-700 hover:underline dark:text-zinc-300"
            target="_blank"
            rel="noopener noreferrer"
          >
            or build from source
          </a>
        </div>

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

      {/* Open Source */}
      <motion.section
        variants={VARIANTS_SECTION}
        transition={TRANSITION_SECTION}
        className="mb-12"
      >
        <h2 className="mb-6 text-2xl font-bold">Open Source & Community</h2>

        <p className="mb-4 text-zinc-600 dark:text-zinc-400">
          Holdem is completely free and open-source.
          <br />
          Built with ❤️ by{' '}
          <a
            href="https://github.com/iamzubin"
            className="text-black hover:underline dark:text-white"
          >
            @iamzubin
          </a>
          . Contributions and feedback are always welcome.
        </p>
      </motion.section>
    </motion.main>
  )
}
