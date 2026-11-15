import { type ClassValue, clsx } from "clsx"
import { twMerge } from "tailwind-merge"

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

const SIZE_UNITS = ['byte', 'kilobyte', 'megabyte', 'gigabyte', 'terabyte'] as const;

/** Human-readable size with localized units (e.g. "1.5 MB", "1,5 Mo"). */
export const formatFileSize = (bytes: number, locale?: string): string => {
  const value = Number.isFinite(bytes) && bytes > 0 ? bytes : 0;
  const i =
    value === 0 ? 0 : Math.min(SIZE_UNITS.length - 1, Math.floor(Math.log(value) / Math.log(1024)));
  return new Intl.NumberFormat(locale, {
    style: 'unit',
    unit: SIZE_UNITS[i],
    unitDisplay: 'short',
    maximumFractionDigits: 1,
  }).format(value / Math.pow(1024, i));
};