import type { Metadata, Viewport } from 'next'
import { Geist, Geist_Mono } from 'next/font/google'
import './globals.css'
import { Header } from './header'
import { Footer } from './footer'
import { ThemeProvider } from 'next-themes'
import { Analytics } from "@vercel/analytics/react"

export const viewport: Viewport = {
  width: 'device-width',
  initialScale: 1,
  themeColor: '#ffffff',
}

export const metadata: Metadata = {
  title: 'Holdem - A simpler way to manage your files',
  description:
    "Holdem is a lightweight, open-source tool for effortlessly dragging and organizing files on your desktop.",
}

const geist = Geist({
  variable: '--font-geist',
  subsets: ['latin'],
})

const geistMono = Geist_Mono({
  variable: '--font-geist-mono',
  subsets: ['latin'],
})

async function getGitHubStars() {
  try {
    const res = await fetch('https://api.github.com/repos/iamzubin/holdem', { next: { revalidate: 3600 } })
    const data = await res.json()
    return data.stargazers_count || 0
  } catch {
    return 0
  }
}

export default async function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode
}>) {
  const stars = await getGitHubStars()
  return (
    <html lang="en" suppressHydrationWarning>
      <body
        className={`${geist.variable} ${geistMono.variable} bg-white tracking-tight antialiased dark:bg-zinc-950`}
      >
        <Analytics />
        {/* GitHub Star Floating Button */}
        <a
          href="https://github.com/iamzubin/holdem"
          target="_blank"
          rel="noopener noreferrer"
          className="fixed z-50"
          style={{
            top: '20px',
            right: '20px',
            position: 'fixed',
            display: 'block',
          }}
        >
          <span className="px-5 py-2 rounded-lg font-medium flex items-center gap-2 shadow transition-colors bg-zinc-100 text-zinc-900 hover:bg-zinc-200 dark:bg-zinc-800 dark:text-zinc-100 dark:hover:bg-zinc-700 border border-transparent" style={{ height: '48px' }}>
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
              <path d="M12 2L14.7553 8.51147L21.9021 9.23607L16.4511 13.9885L18.1803 21.0139L12 17.5L5.81966 21.0139L7.54894 13.9885L2.09789 9.23607L9.24472 8.51147L12 2Z" fill="#FFD600" stroke="#FFD600" strokeWidth="1.5" strokeLinejoin="round"/>
            </svg>
            <span className="font-bold">Star on Github</span>
            <span className="ml-1 font-semibold text-base">{stars}</span>
          </span>
        </a>
        <ThemeProvider
          enableSystem={true}
          attribute="class"
          storageKey="theme"
          defaultTheme="system"
        >
          <div className="flex min-h-screen w-full flex-col font-[family-name:var(--font-inter-tight)]">
            <div className="relative mx-auto w-full max-w-screen-sm flex-1 px-4 pt-20">
              {/* <Header /> */}
              {children}
              <Footer />
            </div>
          </div>
        </ThemeProvider>
      </body>
    </html>
  )
}
