/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_APP_LATEST_RELEASE_NOTES_URL?: string;
  readonly VITE_APP_FULL_CHANGELOG_URL?: string;
}

declare module "*.vue" {
  import type { DefineComponent } from "vue";
  const component: DefineComponent<{}, {}, any>;
  export default component;
}
