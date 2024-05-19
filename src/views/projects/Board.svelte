<script lang="ts">
  import { onMount } from "svelte";
  import { z, string, literal, object } from "zod";

  import type { BaseUrl, Project } from "@httpd-client";

  import config from "virtual:config";
  import { httpdStore } from "@app/lib/httpd";
  import * as utils from "@app/lib/utils";
  import * as role from "@app/lib/roles";
  import { theme, type Theme } from "@app/lib/appearance";

  import ErrorMessage from "@app/components/ErrorMessage.svelte";
  import Loading from "@app/components/Loading.svelte";
  import Layout from "./Layout.svelte";

  export let baseUrl: BaseUrl;
  export let project: Project;

  const RpbBoardsConfigSchema = object({
    enabled: literal(true),
    origin: string().url(),
  });
  type RpbConfig = z.infer<typeof RpbBoardsConfigSchema>;

  const incomingMessageSchema = object({
    type: literal("request-auth-token"),
  });

  type OutgoingMessage =
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

  let loading = true;
  let error: any;
  let rpbConfig: RpbConfig | undefined;
  let iFrameSrc: string;
  let iFrame: HTMLIFrameElement;

  $: session =
    $httpdStore.state === "authenticated" && utils.isLocal(baseUrl.hostname)
      ? $httpdStore.session
      : undefined;
  $: delegates = project.delegates.map(d => d.id);

  function postMessage(message: OutgoingMessage) {
    if (!rpbConfig) {
      return;
    }

    iFrame?.contentWindow?.postMessage(message, rpbConfig.origin);
  }

  function handleIncomingMessage(event: MessageEvent) {
    if (!rpbConfig || event.origin !== rpbConfig.origin) {
      return;
    }

    const result = incomingMessageSchema.safeParse(event.data);
    if (!result.success) {
      return;
    }

    switch (result.data.type) {
      case "request-auth-token":
        if (session?.id) {
          postMessage({ type: "set-auth-token", authToken: session.id });
        }
        break;
    }
  }

  onMount(() => {
    try {
      rpbConfig = RpbBoardsConfigSchema.parse(
        config.plugins?.radiclePlanningBoards,
      );

      const url = new URL(rpbConfig.origin);
      url.pathname = `${baseUrl.hostname}:${baseUrl.port}/${project.id}`;
      url.searchParams.set("initialTheme", $theme);
      url.searchParams.set("baseUrl", window.location.origin);
      url.searchParams.set(
        "canEditLabels",
        (!!role.isDelegate(session?.publicKey, delegates)).toString(),
      );
      iFrameSrc = url.toString();
    } catch (e) {
      error = e;
    }
  });

  $: postMessage({ type: "theme", theme: $theme });
  $: {
    if (!loading) {
      if (session?.id) {
        postMessage({ type: "set-auth-token", authToken: session.id });
      } else {
        postMessage({ type: "remove-auth-token" });
      }
    }
  }
</script>

<style>
  iframe {
    display: block;
    width: 100%;
    height: 100%;
  }

  .hidden {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border-width: 0;
  }
</style>

<svelte:window on:message={handleIncomingMessage} />

<Layout {baseUrl} {project} activeTab="board">
  {#if error}
    <ErrorMessage
      title="Invalid configuration"
      description="Check the configuration for radicle-planning-boards"
      {error} />
  {:else}
    <iframe
      bind:this={iFrame}
      title="Planning Board"
      src={iFrameSrc}
      frameborder="0"
      class:hidden={loading}
      on:load={() => {
        loading = false;
      }}
      allow="clipboard-read; clipboard-write">
    </iframe>

    {#if loading}
      <Loading center />
    {/if}
  {/if}
</Layout>
