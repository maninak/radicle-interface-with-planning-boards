import { defineConfig } from "vite";
import path from "node:path";

export default defineConfig({
  test: {
    environment: "happy-dom",
    include: ["httpd-client/tests/*.test.ts"],
    reporters: "verbose",
    globalSetup: "./tests/support/globalSetup",
  },
  resolve: {
    alias: {
      "@tests": path.resolve("./tests"),
      "@app": path.resolve("./src"),
      "@httpd-client": path.resolve("./httpd-client"),
    },
  },
});
