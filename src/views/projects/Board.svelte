<script lang="ts">
  import type { BaseUrl, Project } from "@httpd-client";
  import Layout from "./Layout.svelte";

  export let baseUrl: BaseUrl;
  export let project: Project;
</script>

<Layout {baseUrl} {project} activeTab="board">
  <span>Planning Board</span>
</Layout>
