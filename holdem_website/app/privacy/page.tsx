import type { Metadata } from 'next'
import Link from 'next/link'
import type { ReactNode } from 'react'

export const metadata: Metadata = {
  title: 'Privacy Policy | Holdem',
  description:
    'What Holdem keeps on your computer, the optional usage data it sends, and what the website measures.',
  alternates: {
    canonical: 'https://holdem.iamzub.in/privacy',
  },
}

const ISSUES_URL = 'https://github.com/iamzubin/holdem/issues/new'

const EVENTS = [
  'app started, with the number of items restored',
  'shelf shown, and how it was opened: shake, hotkey, edge peek, or tray',
  'item list opened',
  'settings opened',
  'files added, with a count',
  'items dragged out, with a count and whether they were dropped somewhere',
  'items removed, with a count',
  'shelf cleared, with a count',
  "a drop that couldn't be added, with the reason and a count",
  'hotkey registered',
  'usage data turned on',
  'start with Windows turned on or off',
  'update checked, with whether one is available (the version downloaded from the website only)',
]

function ExternalLink({
  href,
  children,
}: {
  href: string
  children: ReactNode
}) {
  return (
    <a
      href={href}
      target="_blank"
      rel="noopener noreferrer"
      className="underline"
    >
      {children}
    </a>
  )
}

function Section({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section>
      <h2 className="mt-10 mb-4 text-2xl font-bold">{title}</h2>
      <div className="text-ink-muted space-y-4">{children}</div>
    </section>
  )
}

export default function PrivacyPage() {
  return (
    <main className="mx-auto max-w-3xl py-8">
      <Link href="/" className="text-ink-subtle hover:text-ink text-sm">
        ← Back to Holdem
      </Link>
      <p className="text-ink-subtle mt-6 text-sm font-medium tracking-widest uppercase">
        Privacy
      </p>
      <h1 className="mb-3 text-3xl font-bold md:text-4xl">Privacy policy</h1>
      <p className="text-ink-subtle text-sm">Last updated November 17, 2026</p>
      <p className="text-ink-muted mt-4">
        Holdem is made by Zubin Choudhary. It needs no account, and your files
        never leave your computer.
      </p>

      <Section title="What Holdem keeps on your computer">
        <p>
          Holdem keeps a list of the items on each shelf so they survive a
          restart. It stores settings on your computer. A dropped file stays
          where it is. Holdem only remembers its location.
        </p>
        <p>
          Holdem saves content that exists only on the shelf in its app-data
          folder. That covers text snippets, images from web pages, email
          attachments, and files from ZIP folders and archive apps.
        </p>
        <p>
          When you remove one of these saved items, Holdem keeps it in that
          folder for 7 days, then moves it to the Recycle Bin. Holdem asks
          before deleting anything that can&apos;t go to the Recycle Bin.
        </p>
      </Section>

      <Section title="Usage data is optional">
        <p>
          Usage data is off unless you agree. A window asks on first start.
          Change it any time in Settings with the &quot;Share usage data&quot;
          switch.
        </p>
        <p>
          When usage data is on, Holdem sends these events to{' '}
          <ExternalLink href="https://posthog.com/privacy">
            PostHog
          </ExternalLink>
          , an analytics service, and no others:
        </p>
        <ul className="list-disc space-y-1 pl-6">
          {EVENTS.map((event) => (
            <li key={event}>{event}</li>
          ))}
        </ul>
        <p>
          Each event carries the Holdem version and a random ID created on your
          computer at install. Events never include file names, paths, file
          contents, or anything you typed.
        </p>
        <p>
          Holdem turns off PostHog&apos;s IP-based location lookup and tells
          PostHog not to create a person profile. The connection, like any
          internet request, reaches PostHog&apos;s servers from your IP address.
          Events go over HTTPS to PostHog&apos;s US cloud.
        </p>
        <p>
          The developer uses these events to see which features people use, how
          the shelf gets opened, and why drops fail, to decide what to fix and
          build next.
        </p>
        <p>
          Turning usage data off stops all events at once. Events already sent
          stay in PostHog, tied only to the random ID, which isn&apos;t linked
          to your name, email, or account.
        </p>
      </Section>

      <Section title="Internet use">
        <p>Holdem uses the internet only for these three things:</p>
        <ul className="list-disc space-y-2 pl-6">
          <li>Usage data, only if you turn it on.</li>
          <li>
            Downloading an image when you drop a link that points straight to
            one, from that address.
          </li>
          <li>
            Updates. The Microsoft Store updates the Store version. The website
            version checks holdem.iamzub.in and GitHub for updates, and asks
            before installing one.
          </li>
        </ul>
      </Section>

      <Section title="Who gets your data">
        <p>
          Holdem sends data only to PostHog. It doesn&apos;t sell data and has
          no ads.
        </p>
      </Section>

      <Section title="Logs">
        <p>
          Holdem writes a log file for troubleshooting. It stays on your
          computer unless you attach it to a problem report.
        </p>
      </Section>

      <Section title="Microsoft Store">
        <p>
          Microsoft handles download, updates, and any payment under the{' '}
          <ExternalLink href="https://privacy.microsoft.com/privacystatement">
            Microsoft Privacy Statement
          </ExternalLink>
          . Holdem receives no payment details.
        </p>
      </Section>

      <Section title="The Holdem website">
        <p>
          holdem.iamzub.in counts visits with Vercel Web Analytics and Google
          Analytics. Google Analytics uses cookies. They record pages viewed,
          the referring site, browser, device type, and country.
        </p>
      </Section>

      <Section title="Children">
        <p>
          Holdem is not directed at children and doesn&apos;t knowingly collect
          their data.
        </p>
      </Section>

      <Section title="Changes to this policy">
        <p>The date at the top changes when this policy does.</p>
      </Section>

      <Section title="Contact">
        <p>
          Ask about this policy by{' '}
          <ExternalLink href={ISSUES_URL}>
            opening an issue on GitHub
          </ExternalLink>
          .
        </p>
      </Section>
    </main>
  )
}
