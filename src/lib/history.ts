import { del, get, set } from 'idb-keyval';

const HISTORY_KEY = 'easytube-history';

export interface HistoryEntry {
  jobId: string;
  url: string;
  title: string;
  formatId?: string;
  resolution?: string;
  outputFile?: string;
  status: 'done' | 'failed' | 'cancelled';
  downloadedAt: string;
}

export async function getHistory(): Promise<HistoryEntry[]> {
  const entries = await get<HistoryEntry[]>(HISTORY_KEY);
  return entries ?? [];
}

export async function addHistoryEntry(entry: HistoryEntry): Promise<void> {
  const entries = await getHistory();
  entries.unshift(entry);
  if (entries.length > 1000) {
    entries.length = 1000;
  }
  await set(HISTORY_KEY, entries);
}

export async function removeHistoryEntry(jobId: string): Promise<void> {
  const entries = await getHistory();
  await set(
    HISTORY_KEY,
    entries.filter((e) => e.jobId !== jobId),
  );
}

export async function clearHistory(): Promise<void> {
  await del(HISTORY_KEY);
}

export async function hasDownloaded(url: string): Promise<boolean> {
  const entries = await getHistory();
  return entries.some((e) => e.url === url && e.status === 'done');
}

export async function checkDuplicateDownload(
  url: string,
  formatId: string,
): Promise<HistoryEntry | null> {
  const entries = await getHistory();
  return (
    entries.find(
      (e) => e.url === url && e.formatId === formatId && e.status === 'done',
    ) ?? null
  );
}
