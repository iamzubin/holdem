import * as React from 'react'
import { cn } from '@/lib/utils'

export type ButtonVariant = 'primary' | 'secondary' | 'ghost'
export type ButtonSize = 'sm' | 'md' | 'lg'

// Linear button spec: 8px corners, 14px/500 labels, no shadows.
// Primary is lavender; secondary is a charcoal hairline button.
const base =
  'inline-flex items-center justify-center gap-2 rounded-md font-medium transition duration-150 ease-out active:scale-[0.97] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary-focus/50 disabled:pointer-events-none disabled:opacity-50'

const variants: Record<ButtonVariant, string> = {
  primary: 'bg-primary text-white hover:bg-primary-hover',
  secondary: 'border border-hairline bg-surface-1 text-ink hover:bg-surface-2',
  ghost: 'text-ink hover:bg-surface-1',
}

const sizes: Record<ButtonSize, string> = {
  sm: 'px-3 py-1.5 text-sm',
  md: 'px-3.5 py-2 text-sm',
  lg: 'px-4 py-2.5 text-sm',
}

export function buttonClass({
  variant = 'primary',
  size = 'md',
  className,
}: {
  variant?: ButtonVariant
  size?: ButtonSize
  className?: string
} = {}) {
  return cn(base, variants[variant], sizes[size], className)
}

export interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant
  size?: ButtonSize
}

const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  (
    { className, variant = 'primary', size = 'md', type = 'button', ...props },
    ref,
  ) => {
    return (
      <button
        type={type}
        className={buttonClass({ variant, size, className })}
        ref={ref}
        {...props}
      />
    )
  },
)
Button.displayName = 'Button'

export interface ButtonLinkProps extends React.AnchorHTMLAttributes<HTMLAnchorElement> {
  variant?: ButtonVariant
  size?: ButtonSize
}

const ButtonLink = React.forwardRef<HTMLAnchorElement, ButtonLinkProps>(
  ({ className, variant = 'primary', size = 'md', ...props }, ref) => {
    return (
      <a
        className={buttonClass({ variant, size, className })}
        ref={ref}
        {...props}
      />
    )
  },
)
ButtonLink.displayName = 'ButtonLink'

export { Button, ButtonLink }
