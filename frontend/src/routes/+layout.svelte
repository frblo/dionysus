<script lang="ts">
  import "../app.css";
  import favicon from "$lib/assets/favicon.svg";
  import type { LayoutData } from "./$types";
  import { sessionState } from "$lib/state/session.svelte";

  let { data, children }: { data: LayoutData; children: any } = $props();

  $effect(() => {
    sessionState.current = data.session;
  });
</script>

<svelte:head>
  <link rel="icon" href={favicon} />
  <title>Dionysus</title>
</svelte:head>

<div
  class="fixed inset-0 flex flex-col overflow-hidden bg-[#1e1e1e] text-gray-200"
>
  {@render children()}
</div>
