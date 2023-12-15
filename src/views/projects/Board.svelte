<script lang="ts">
  import type { BaseUrl, Project } from "@httpd-client";
  import Layout from "./Layout.svelte";

  import { theme, type Theme } from "@app/lib/appearance";

  const RPB_BASE_URL = "https://chipper-wisp-c7553e.netlify.app";

  interface RPBMessage {
    type: "theme";
    theme: Theme;
  }

  export let baseUrl: BaseUrl;
  export let project: Project;
  export let seeding: boolean;

  let iFrameSrc: string;
  let iFrame: HTMLIFrameElement;
  let isIFrameLoading = true;

  $: {
    iFrame?.contentWindow?.postMessage(
      { type: "theme", theme: $theme } satisfies RPBMessage,
      RPB_BASE_URL,
    );
  }

  const originalTheme = $theme;
  $: iFrameSrc = `${RPB_BASE_URL}/${baseUrl.hostname}:${baseUrl.port}/${project.id}?initialTheme=${originalTheme}`;
</script>

<style>
  .board {
    height: 75vh;
  }
</style>

<Layout {baseUrl} {project} {seeding} activeTab="board">
  <div class="board">
    <iframe
      bind:this={iFrame}
      title="Planning Board"
      src={iFrameSrc}
      width="100%"
      height="100%"
      frameborder="0"
      style:visibility={isIFrameLoading ? "hidden" : "visible"}
      on:load={() => {
        isIFrameLoading = false;
      }}>
    </iframe>
  </div>
</Layout>
