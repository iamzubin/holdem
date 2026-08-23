import Link from 'next/link'

export default function VsIndex() {
  return (
    <main className="mx-auto max-w-3xl py-8">
      <Link href="/" className="text-sm text-zinc-500 hover:text-zinc-900 dark:hover:text-zinc-100">
        ← Back to Holdem
      </Link>
      <h1 className="mt-6 text-2xl font-bold">Compare Holdem</h1>
      <p className="mt-2 text-zinc-600 dark:text-zinc-400">See how Holdem stacks up against popular alternatives.</p>
      <div className="mt-6 grid gap-3">
        <Link href="/vs/dropshelf" className="rounded-xl border border-zinc-200 p-4 hover:bg-zinc-50 dark:border-zinc-800 dark:hover:bg-zinc-900">
          <div className="font-medium">Holdem vs Dropshelf</div>
          <div className="text-sm text-zinc-500">The best free Dropshelf alternative — no 3-second wait, 100% open source.</div>
        </Link>
        <Link href="/vs/droppoint" className="rounded-xl border border-zinc-200 p-4 hover:bg-zinc-50 dark:border-zinc-800 dark:hover:bg-zinc-900">
          <div className="font-medium">Holdem vs DropPoint</div>
          <div className="text-sm text-zinc-500">Tauri/Rust vs Electron — clean, minimal vs dated &amp; clunky. Modern upgrade.</div>
        </Link>
        <Link href="/vs/dropover" className="rounded-xl border border-zinc-200 p-4 hover:bg-zinc-50 dark:border-zinc-800 dark:hover:bg-zinc-900">
          <div className="font-medium">Holdem vs Dropover</div>
          <div className="text-sm text-zinc-500">Free Dropover alternative for Windows — coming soon.</div>
        </Link>
      </div>
    </main>
  )
}
