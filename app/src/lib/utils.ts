import { type ClassValue, clsx } from "clsx"
import { twMerge } from "tailwind-merge"

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

const SIZE_UNITS = ['byte', 'kilobyte', 'megabyte', 'gigabyte', 'terabyte'] as const;

/** Human-readable size with localized units (e.g. "1.5 MB", "1,5 Mo"). */
export const formatFileSize = (bytes: number, locale?: string): string => {
  const value = Number.isFinite(bytes) && bytes > 0 ? bytes : 0;
  let i =
    value === 0 ? 0 : Math.min(SIZE_UNITS.length - 1, Math.floor(Math.log(value) / Math.log(1000)));
  // 999,960 bytes would round to "1,000 kB": show it as "1 MB" instead.
  if (i > 0 && i < SIZE_UNITS.length - 1 && Math.round((value / 1000 ** i) * 10) / 10 >= 1000) {
    i += 1;
  }
  return new Intl.NumberFormat(locale, {
    style: 'unit',
    unit: SIZE_UNITS[i],
    // "short" gives "660 byte" for plain bytes in English; spell those out.
    unitDisplay: i === 0 ? 'long' : 'short',
    maximumFractionDigits: 1,
  }).format(value / Math.pow(1000, i));
};