import { createServerRequestError } from '@/utils/api';

export const APP_LATEST_RELEASE_NOTES_URL =
  import.meta.env.VITE_APP_LATEST_RELEASE_NOTES_URL ??
  'https://raw.githubusercontent.com/ParadiseZ/AutoDailyTauriRelease/main/LATEST_RELEASE.md';

export const APP_FULL_CHANGELOG_URL =
  import.meta.env.VITE_APP_FULL_CHANGELOG_URL ??
  'https://raw.githubusercontent.com/ParadiseZ/AutoDailyTauriRelease/main/CHANGELOG.md';

export const fetchMarkdownDocument = async (url: string) => {
  try {
    const response = await fetch(url, { cache: 'no-store' });
    if (!response.ok) {
      throw createServerRequestError(`fetch:${url}`, `HTTP ${response.status} ${response.statusText}`, '加载更新日志失败，请稍后重试。');
    }
    return response.text();
  } catch (error) {
    if (error instanceof Error && error.message === '加载更新日志失败，请稍后重试。') {
      throw error;
    }

    throw createServerRequestError(`fetch:${url}`, error, '加载更新日志失败，请稍后重试。');
  }
};
