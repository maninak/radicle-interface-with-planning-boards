import type { BaseUrl } from "@httpd-client";

import configJson from "@app/config.json";
import { merge } from "lodash";
import { object, string, number, unknown, boolean, record } from "zod";

export interface Config {
  nodes: {
    apiVersion: string;
    fallbackPublicExplorer: string;
    defaultHttpdPort: number;
    defaultHttpdHostname: string;
    defaultLocalHttpdPort: number;
    defaultNodePort: number;
    defaultHttpdScheme: string;
    pinned: { baseUrl: BaseUrl }[];
  };
  supportWebsite: string;
  fallbackPreferredSeed: BaseUrl;
  plugins?: Record<
    string,
    {
      enabled: boolean;
      [key: string]: unknown;
    }
  >;
}

const baseUrlSchema = object({
  hostname: string(),
  port: number(),
  scheme: string(),
});

const configSchema = object({
  nodes: object({
    apiVersion: string(),
    fallbackPublicExplorer: string(),
    defaultHttpdPort: number(),
    defaultHttpdHostname: string(),
    defaultLocalHttpdPort: number(),
    defaultNodePort: number(),
    defaultHttpdScheme: string(),
    pinned: object({ baseUrl: baseUrlSchema }).array(),
  }),
  supportWebsite: string(),
  fallbackPreferredSeed: baseUrlSchema,
  plugins: record(
    string(),
    object({ enabled: boolean() }).and(record(string(), unknown())),
  ).optional(),
});

function getEnvConfig(): Config {
  const {
    VITE_CONFIG_NODES: nodes,
    VITE_CONFIG_SUPPORT_WEBSITE: supportWebsite,
    VITE_CONFIG_FALLBACK_PREFERRED_SEED: fallbackPreferredSeed,
    VITE_CONFIG_PLUGINS: plugins,
  } = import.meta.env;

  try {
    const envConfig = configSchema.deepPartial().parse({
      nodes: nodes ? JSON.parse(nodes) : undefined,
      supportWebsite,
      fallbackPreferredSeed: fallbackPreferredSeed
        ? JSON.parse(fallbackPreferredSeed)
        : undefined,
      plugins: plugins ? JSON.parse(plugins) : undefined,
    });

    const config = merge(configJson, envConfig);

    return config;
  } catch (error) {
    console.error(
      "Error parsing config from environment variables. Using config.json.",
      error,
    );

    return configJson;
  }
}

function getConfig(): Config {
  if (window.VITEST) {
    return {
      nodes: {
        fallbackPublicExplorer: "https://app.radicle.xyz/nodes/$host/$rid$path",
        apiVersion: "0.1.0",
        defaultHttpdHostname: "127.0.0.1",
        defaultHttpdPort: 8081,
        defaultLocalHttpdPort: 8081,
        defaultHttpdScheme: "http",
        defaultNodePort: 8776,
        pinned: [],
      },
      supportWebsite: "https://radicle.zulipchat.com",
      fallbackPreferredSeed: {
        hostname: "seed.radicle.garden",
        port: 443,
        scheme: "https",
      },
    };
  } else if (window.PLAYWRIGHT) {
    return window.APP_CONFIG;
  } else {
    // In dev and production environments we use data from config.json.
    return getEnvConfig();
  }
}

export const config = getConfig();
