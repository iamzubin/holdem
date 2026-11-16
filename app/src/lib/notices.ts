import type { TFunction } from 'i18next';

/** Why a dropped item was not added. Mirrors `DropIssue` in drop_report.rs. */
export type DropIssue =
  | 'unsupported'
  | 'tooLarge'
  | 'dropTooLarge'
  | 'noContents'
  | 'readFailed'
  | 'savedAsLink'
  | 'diskFull'
  | 'accessDenied'
  | 'saveFailed'
  | 'unreadable'
  | 'alreadyOnShelf'
  | 'elevatedSource'
  | 'holdemElevated'
  | 'clipboardBusy';

/** Size limits, as drop_report.rs sends them. */
interface Limits {
  itemMb: number;
  dropMb: number;
  files: number;
}

/** Payload of the backend's `shelf-notice` event. */
export type ShelfNotice =
  | { kind: 'missing'; count: number }
  | {
      kind: 'drop';
      report: {
        issues: { reason: DropIssue; count: number }[];
        name?: string | null;
        /** Items from the same drop that were added. */
        added: number;
      };
      limits: Limits;
    };

export interface NoticeText {
  /** Fits the small shelf. */
  short: string;
  /** Every reason as a sentence, for the item list and the shelf's tooltip. */
  full: string;
  /** Something failed (not just "already there" or "kept as a link"). */
  error: boolean;
}

/** Most important first: the shelf only has room for the first one. */
const ORDER: DropIssue[] = [
  'holdemElevated',
  'elevatedSource',
  'diskFull',
  'accessDenied',
  'saveFailed',
  'tooLarge',
  'dropTooLarge',
  'noContents',
  'readFailed',
  'unreadable',
  'unsupported',
  'clipboardBusy',
  'savedAsLink',
  'alreadyOnShelf',
];
const INFO: DropIssue[] = ['savedAsLink', 'alreadyOnShelf'];
/** Issues whose sentence can name the one item it is about. */
const NAMED: DropIssue[] = ['tooLarge', 'noContents', 'readFailed', 'unreadable', 'alreadyOnShelf'];
/** Issues whose short text says how many items it is about. */
const COUNTED: DropIssue[] = ['tooLarge', 'noContents', 'unreadable', 'savedAsLink', 'alreadyOnShelf'];

export function shelfNoticeText(notice: ShelfNotice, t: TFunction): NoticeText {
  if (notice.kind === 'missing') {
    const text = t('app.missing', { count: notice.count });
    return { short: text, full: text, error: false };
  }
  const { name, added } = notice.report;
  const { limits } = notice;
  if (notice.report.issues.length === 0) {
    const text = t('app.files', { count: added });
    return { short: text, full: text, error: false };
  }
  const issues = [...notice.report.issues].sort(
    (a, b) => ORDER.indexOf(a.reason) - ORDER.indexOf(b.reason),
  );
  const values = (count: number) => ({
    count,
    name,
    limit: limits.itemMb,
    dropLimit: limits.dropMb,
    files: limits.files,
  });
  const sentences = issues.map(({ reason, count }) =>
    name && NAMED.includes(reason)
      ? t(`drop.named.${reason}`, values(count))
      : t(`drop.full.${reason}`, values(count)),
  );
  const [first] = issues;
  let short = COUNTED.includes(first.reason)
    ? t(`drop.short.${first.reason}`, { count: first.count })
    : t(`drop.short.${first.reason}`);
  // How many other items had a problem (not how many other reasons).
  const others = issues.slice(1).reduce((sum, issue) => sum + issue.count, 0);
  if (others > 0) short = t('drop.andMore', { text: short, count: others });
  // "1 already on the shelf" alone would read as "nothing was added".
  if (added > 0) short = t('drop.addedAnd', { text: short, count: added });
  return {
    short,
    full: sentences.join(' '),
    error: issues.some((issue) => !INFO.includes(issue.reason)),
  };
}
