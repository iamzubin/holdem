import type { Metadata, Viewport } from 'next'
import { Geist, Geist_Mono } from 'next/font/google'
import './globals.css'
import { Footer } from './footer'
import { ThemeProvider } from 'next-themes'
import { Analytics } from '@vercel/analytics/react'
import { GoogleAnalytics } from '@next/third-parties/google'
import { buttonClass } from './components/ui/button'

// GA4 Measurement ID — override with NEXT_PUBLIC_GA_MEASUREMENT_ID env var.
const GA_MEASUREMENT_ID =
  process.env.NEXT_PUBLIC_GA_MEASUREMENT_ID || 'G-9T07RLQTXG'

export const viewport: Viewport = {
  width: 'device-width',
  initialScale: 1,
  themeColor: '#ffffff',
}

export const metadata: Metadata = {
  metadataBase: new URL('https://holdem.iamzub.in'),
  title: {
    default: 'Holdem - Free Dropover for Windows',
    template: '%s | Holdem',
  },
  description:
    'Holdem is the free Dropover for Windows: a free, open-source drag-and-drop file shelf. Shake your mouse while dragging to summon a floating holding area for files, folders & web images. No wait timers, no paywall.',
  // Google Search Console: set NEXT_PUBLIC_GOOGLE_SITE_VERIFICATION in Vercel
  // env vars to emit the google-site-verification meta tag.
  verification: process.env.NEXT_PUBLIC_GOOGLE_SITE_VERIFICATION
    ? { google: process.env.NEXT_PUBLIC_GOOGLE_SITE_VERIFICATION }
    : undefined,
  keywords: [
    'Holdem',
    'drag and drop shelf Windows',
    'free Dropover alternative Windows',
    'Dropshelf alternative',
    'DropPoint alternative',
    'file shelf Windows',
    'drag drop utility',
  ],
  alternates: {
    canonical: 'https://holdem.iamzub.in',
  },
  robots: {
    index: true,
    follow: true,
  },
  openGraph: {
    title: 'Holdem - Free Dropover for Windows',
    description:
      'Free, open-source shelf for Windows: shake to summon, stash files, drop anywhere. No wait timers, no paywall.',
    url: 'https://holdem.iamzub.in',
    siteName: 'Holdem',
    images: [
      {
        url: '/og.png',
        width: 1280,
        height: 640,
        alt: 'Holdem — free drag-and-drop file shelf for Windows, showing floating shelf holding 6 files',
      },
    ],
    type: 'website',
  },
  twitter: {
    card: 'summary_large_image',
    site: '@iamzub_in',
    creator: '@iamzub_in',
    title: 'Holdem - Free Dropover for Windows',
    description:
      'Free, open-source shelf for Windows: shake to summon, stash files, drop anywhere.',
    images: ['/og.png'],
  },
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
    const res = await fetch('https://api.github.com/repos/iamzubin/holdem', {
      next: { revalidate: 3600 },
    })
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
  const softwareSchema = {
    '@context': 'https://schema.org',
    '@type': 'SoftwareApplication',
    name: 'Holdem',
    applicationCategory: 'UtilitiesApplication',
    operatingSystem: 'Windows 10, Windows 11',
    offers: { '@type': 'Offer', price: '0', priceCurrency: 'USD' },
    isAccessibleForFree: true,
    softwareVersion: '3.1.0',
    url: 'https://holdem.iamzub.in',
    downloadUrl:
      'https://github.com/iamzubin/holdem/releases/download/3.1.0/holdem_3.1.0_x64-setup.exe',
    sameAs: ['https://github.com/iamzubin/holdem'],
    description:
      'Free, open-source drag-and-drop file shelf for Windows. Shake your mouse while dragging to summon a floating holding area.',
    author: {
      '@type': 'Person',
      name: 'Zubin Choudhary',
      url: 'https://iamzub.in',
    },
  }
  return (
    <html lang="en" suppressHydrationWarning>
      <head>
        <script
          type="application/ld+json"
          dangerouslySetInnerHTML={{ __html: JSON.stringify(softwareSchema) }}
        />
      </head>
      <body
        className={`${geist.variable} ${geistMono.variable} bg-white tracking-tight antialiased dark:bg-zinc-950`}
      >
        <Analytics />
        <GoogleAnalytics gaId={GA_MEASUREMENT_ID} />
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
          <span
            className={buttonClass({
              variant: 'secondary',
              size: 'md',
              className: 'h-12',
            })}
          >
            <svg
              width="20"
              height="20"
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
            <span className="ml-1 text-base font-semibold">{stars}</span>
          </span>
        </a>
        <ThemeProvider
          enableSystem={true}
          attribute="class"
          storageKey="theme"
          defaultTheme="system"
        >
          <div className="flex min-h-screen w-full flex-col font-[family-name:var(--font-inter-tight)]">
            <div className="relative mx-auto w-full max-w-[90vw] flex-1 px-4 pt-20">
              {children}
              <Footer />
            </div>
          </div>
        </ThemeProvider>
      </body>
    </html>
  )
}
