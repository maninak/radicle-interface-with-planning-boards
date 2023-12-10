<script lang="ts">
  import type { BaseUrl, Project } from "@httpd-client";
  import Layout from "./Layout.svelte";

  import { theme, type Theme } from "@app/lib/appearance";

  interface RPBMessage {
    type: "theme";
    theme: Theme;
  }

  export let baseUrl: BaseUrl;
  export let project: Project;
  export let tracking: boolean;

  let iframe: HTMLIFrameElement;

  $: {
    iframe?.contentWindow?.postMessage(
      { type: "theme", theme: $theme } satisfies RPBMessage,
      "*",
    );
  }

  const originalTheme = $theme;
</script>

<style>
  .board {
    height: 75vh;
  }
</style>

<Layout {baseUrl} {project} {tracking} activeTab="board">
  <div class="board">
    <iframe
      bind:this={iframe}
      title="Planning Board"
      src="http://localhost:3000?initialTheme={originalTheme}"
      width="100%"
      height="100%"
      frameborder="0">
    </iframe>
  </div>
</Layout>
