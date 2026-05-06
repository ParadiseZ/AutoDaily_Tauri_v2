export const APP_LATEST_RELEASE_NOTES_URL =
  import.meta.env.VITE_APP_LATEST_RELEASE_NOTES_URL ??
  'https://raw.githubusercontent.com/ParadiseZ/AutoDailyTauriRelease/main/LATEST_RELEASE.md';

export const APP_FULL_CHANGELOG_URL =
  import.meta.env.VITE_APP_FULL_CHANGELOG_URL ??
  'https://raw.githubusercontent.com/ParadiseZ/AutoDailyTauriRelease/main/CHANGELOG.md';

export const fetchMarkdownDocument = async (url: string) => {
  const response = await fetch(url, { cache: 'no-store' });
  if (!response.ok) {
    throw new Error(`获取更新日志失败: ${response.status}`);
  }
  return response.text();
};

export const openExternalUrl = async (url: string) => {
  if (typeof navigator !== 'undefined' && navigator.userAgent.includes('Tauri')) {
    const { openUrl } = await import('@tauri-apps/plugin-opener');
    await openUrl(url);
    return;
  }
  window.open(url, '_blank', 'noopener,noreferrer');
};
