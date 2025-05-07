'use client'
import { motion } from 'motion/react'
import Link from 'next/link'
import Image from 'next/image'
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

const Feature = ({ title, description }: { title: string, description: string }) => (
  <div className="mb-4">
    <p className="text-lg font-medium flex items-center gap-2">
      <span className="text-green-500">✅</span> {title}
    </p>
    <p className="text-zinc-600 dark:text-zinc-400 pl-7">{description}</p>
  </div>
);

const ComingSoonFeature = ({ title, description }: { title: string, description: string }) => (
  <div className="mb-4">
    <p className="text-lg font-medium flex items-center gap-2">
      <span className="text-yellow-500">🚧</span> {title}
    </p>
    <p className="text-zinc-600 dark:text-zinc-400 pl-7">{description}</p>
  </div>
);

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
      className="px-5 py-2 rounded-lg font-medium flex items-center gap-2 shadow transition-colors bg-zinc-100 text-zinc-900 hover:bg-zinc-200 dark:bg-zinc-800 dark:text-zinc-100 dark:hover:bg-zinc-700 border border-transparent"
    >
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path d="M12 2L14.7553 8.51147L21.9021 9.23607L16.4511 13.9885L18.1803 21.0139L12 17.5L5.81966 21.0139L7.54894 13.9885L2.09789 9.23607L9.24472 8.51147L12 2Z" fill="#FFD600" stroke="#FFD600" strokeWidth="1.5" strokeLinejoin="round"/>
      </svg>
      <span className="font-bold">Star on Github</span>
      <span className="ml-1 font-semibold text-base">{stars !== null ? stars : '—'}</span>
    </a>
  )
}

export default function Home() {
  return (
    <motion.main
      className="space-y-20"
      variants={VARIANTS_CONTAINER}
      initial="hidden"
      animate="visible"
    >
      {/* Hero Section */}
      <motion.section
        className="text-center py-12"
        variants={VARIANTS_SECTION}
        transition={TRANSITION_SECTION}
      >
        <h1 className="text-4xl font-bold mb-4">Holdem</h1>
        <p className="text-xl mb-6 text-zinc-600 dark:text-zinc-400">A simpler way to manage your files.</p>
        <p className="text-zinc-600 dark:text-zinc-400 mb-8 max-w-2xl mx-auto">
          Holdem is a lightweight, open-source tool for effortlessly dragging and organizing files on your desktop. 
          Inspired by Dropover and built with Tauri, Holdem lets you collect and drop files in a temporary 
          holding area — no clutter, no hassle.
        </p>

        <div className="flex flex-row items-center justify-center gap-3">
          <a 
            href="https://github.com/iamzubin/holdem/releases/download/0.1.5/holdem_0.1.5_x64-setup.exe" 
            className="px-5 py-2 rounded-lg font-medium flex items-center gap-2 shadow transition-colors bg-zinc-900 text-white hover:bg-zinc-800 dark:bg-white dark:text-zinc-900 dark:hover:bg-zinc-100 border border-transparent dark:border-zinc-200"
            target="_blank"
            rel="noopener noreferrer"
          >
            ⬇️ Download for Windows
          </a>
          <a
            href="https://www.buymeacoffee.com/iamzubin"
            target="_blank"
            rel="noopener noreferrer"
            className="inline-flex items-center px-0 py-0 rounded-lg font-semibold border-none shadow-none bg-transparent hover:bg-transparent"
            style={{ height: '40px' }}
          >
            <img src="https://img.buymeacoffee.com/button-api/?text=Buy me some fries&emoji=🍟&slug=iamzubin&button_colour=353535&font_colour=ffffff&font_family=Lato&outline_colour=ffffff&coffee_colour=FFDD00" alt="Buy me some fries" style={{ height: '40px', borderRadius: '8px' }} />
          </a>
        </div>
        <div className="mt-2 text-center">
          <a 
            href="https://github.com/iamzubin/holdem" 
            className="text-zinc-700 dark:text-zinc-300 hover:underline text-sm"
            target="_blank"
            rel="noopener noreferrer"
          >
            or build from source
          </a>
        </div>

        <div className="mt-12 rounded-xl overflow-hidden shadow-lg max-w-2xl mx-auto">
          <video 
            className="w-full h-auto" 
            autoPlay 
            loop 
            muted 
            playsInline
          >
            <source src="/assets/herovideo.mp4" type="video/mp4" />
            Your browser does not support the video tag.
          </video>
        </div>
      </motion.section>

      {/* How it works */}
      <motion.section
        variants={VARIANTS_SECTION}
        transition={TRANSITION_SECTION}
      >
        <h2 className="text-2xl font-bold mb-6">How it works</h2>
        <h3 className="text-xl font-medium mb-4">Shake. Drop. Done.</h3>
        
        <p className="text-zinc-600 dark:text-zinc-400 mb-8">
          Holdem runs quietly in your system tray. When you need to move files, simply shake your mouse while dragging — 
          a floating shelf appears, ready to hold your files temporarily. Drag them out when you're ready.
        </p>
        
        <h3 className="text-xl font-medium mb-4">Floating Shelf for Your Files</h3>
        
        <p className="text-zinc-600 dark:text-zinc-400 mb-8">
          Keep your files within reach while switching between folders, apps, or desktops. 
          Holdem helps you multitask without losing track of what you're moving.
        </p>
      </motion.section>

      {/* Features */}
      <motion.section
        variants={VARIANTS_SECTION}
        transition={TRANSITION_SECTION}
      >
        <h2 className="text-2xl font-bold mb-6">Features</h2>
        
        <div className="grid md:grid-cols-2 gap-4">
          <div>
            <Feature 
              title="Floating Holding Area" 
              description="Instantly accessible, draggable shelf to drop and hold files." 
            />
            <Feature 
              title="Drag and Drop Support" 
              description="Drag files or folders in and out of the shelf with ease." 
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
        <h2 className="text-2xl font-bold mb-6">Coming Soon</h2>
        
        <div className="grid md:grid-cols-2 gap-4">
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
        <h2 className="text-2xl font-bold mb-6">Open Source & Community</h2>
        
        <p className="text-zinc-600 dark:text-zinc-400 mb-4">
          Holdem is completely free and open-source.<br />
          Built with ❤️ by <a href="https://github.com/iamzubin" className="text-black dark:text-white hover:underline">@iamzubin</a>. Contributions and feedback are always welcome.
        </p>
      </motion.section>
    </motion.main>
  )
}
