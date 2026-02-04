<script lang="ts">
  import "../app.css";
  import favicon from "$lib/assets/favicon.svg";

  let { children } = $props();
</script>

<svelte:head>
  <link rel="icon" href={favicon} />
</svelte:head>

<div
  class="fixed inset-0 flex flex-col overflow-hidden bg-gray-100 bg-[#1e1e1e] text-gray-200"
>
  {@render children()}
</div>
