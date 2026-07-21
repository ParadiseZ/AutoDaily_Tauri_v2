import { DISCLAIMER_ACCEPTANCE_KEY } from '@/constants/legal';
import { getFromStore, setToStore } from '@/store/store';

export const hasAcceptedDisclaimer = async () =>
  (await getFromStore<boolean>(DISCLAIMER_ACCEPTANCE_KEY)) === true;

export const acceptDisclaimer = async () => {
  await setToStore(DISCLAIMER_ACCEPTANCE_KEY, true);
};

export const declineDisclaimer = async () => {
  const { exit } = await import('@tauri-apps/plugin-process');
  await exit(0);
};
