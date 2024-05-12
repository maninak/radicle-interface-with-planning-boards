/* eslint-disable @typescript-eslint/naming-convention */

// Defined by vite or by custom passed env variables
// Custom entries will be defined as globals during dev and statically replaced during build.
interface ImportMeta extends Readonly<Record<string, unknown>> {
  env: {
    MODE: string;
    BASE_URL: string;
    PROD: boolean;
    DEV: boolean;
    SSR: boolean;
    VITE_CONFIG_NODES?: string;
    VITE_CONFIG_SUPPORT_WEBSITE?: string;
    VITE_CONFIG_FALLBACK_PREFERRED_SEED?: string;
    VITE_CONFIG_PLUGINS?: string;
  };
}
