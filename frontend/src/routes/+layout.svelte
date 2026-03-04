<script lang="ts">
  import "../app.css";
  import favicon from "$lib/assets/favicon.svg";

  let { children } = $props();
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
