<script lang="ts">
  import { httpdStore } from "@app/lib/httpd";
  import * as utils from "@app/lib/utils";
  import type { BaseUrl, Project } from "@httpd-client";
  import z from "zod";
  import Layout from "./Layout.svelte";

  import { theme, type Theme } from "@app/lib/appearance";

  const RPB_ORIGIN = "https://chipper-wisp-c7553e.netlify.app";

  type RPBOutgoingMessage =
    | {
        type: "theme";
        theme: Theme;
      }
    | {
        type: "set-auth-token";
        authToken: string;
      }
    | {
        type: "remove-auth-token";
      };

  const incomingMessageSchema = z.object({
    type: z.literal("request-auth-token"),
  });

  export let baseUrl: BaseUrl;
  export let project: Project;
  export let seeding: boolean;

  let iFrameSrc: string;
  let iFrame: HTMLIFrameElement;
  let hasIFrameLoaded = false;

  $: session =
    $httpdStore.state === "authenticated" && utils.isLocal(baseUrl.hostname)
      ? $httpdStore.session
      : undefined;

  function postMessageToRPB(message: RPBOutgoingMessage) {
    iFrame?.contentWindow?.postMessage(message, RPB_ORIGIN);
  }

  function handleIncomingMessage(event: MessageEvent) {
    if (event.origin !== RPB_ORIGIN) {
      return;
    }

    const result = incomingMessageSchema.safeParse(event.data);
    if (
      !result.success ||
      result.data.type !== "request-auth-token" ||
      !session?.id
    ) {
      return;
    }

    postMessageToRPB({ type: "set-auth-token", authToken: session.id });
  }

  $: postMessageToRPB({ type: "theme", theme: $theme });
  $: {
    if (hasIFrameLoaded && session?.id) {
      postMessageToRPB({ type: "set-auth-token", authToken: session.id });
    } else if (hasIFrameLoaded && !session?.id) {
      postMessageToRPB({ type: "remove-auth-token" });
    }
  }

  const originalTheme = $theme;

  $: {
    const url = new URL(RPB_ORIGIN);
    url.pathname = `${baseUrl.hostname}:${baseUrl.port}/${project.id}`;
    url.searchParams.set("initialTheme", originalTheme);
    url.searchParams.set("baseUrl", window.location.origin);
    iFrameSrc = url.toString();
  }
</script>

<style>
  .board {
    height: 75vh;
  }
</style>

<svelte:window on:message={handleIncomingMessage} />

<Layout {baseUrl} {project} {seeding} activeTab="board">
  <div class="board">
    <iframe
      bind:this={iFrame}
      title="Planning Board"
      src={iFrameSrc}
      width="100%"
      height="100%"
      frameborder="0"
      style:visibility={hasIFrameLoaded ? "visible" : "hidden"}
      on:load={() => {
        hasIFrameLoaded = true;
      }}>
    </iframe>
  </div>
</Layout>
