import { cn } from '@/lib/utils'
import { STORE_URL } from '@/lib/constants'

// Microsoft's official "Get it from Microsoft" badge. Their brand guidelines
// ask for the unmodified artwork, so it is loaded from get.microsoft.com
// rather than redrawn.
export function StoreBadge({
  className,
  height = 48,
}: {
  className?: string
  height?: number
}) {
  return (
    <a
      href={STORE_URL}
      target="_blank"
      rel="noopener noreferrer"
      className={cn(
        'inline-flex rounded-lg transition duration-150 ease-out hover:opacity-90 active:scale-[0.97]',
        className,
      )}
    >
      <img
        src="https://get.microsoft.com/images/en-us%20dark.svg"
        alt="Get it from Microsoft"
        width={Math.round(height * 2.88)}
        height={height}
        style={{ height, width: 'auto' }}
      />
    </a>
  )
}
