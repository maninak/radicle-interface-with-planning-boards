<script lang="ts">
  import type { BaseUrl, Project } from "@httpd-client";
  import Layout from "./Layout.svelte";

  export let baseUrl: BaseUrl;
  export let project: Project;
  export let tracking: boolean;
</script>

<style>
</style>

<Layout {baseUrl} {project} {tracking} activeTab="board">
  <span>Radicle planning boards</span>
</Layout>
