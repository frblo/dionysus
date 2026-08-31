<script lang="ts">
  import type { Snippet } from "svelte";
  import { House } from "svelte-bootstrap-icons";

  let {
    title = "",
    children,
  }: {
    title?: string;
    children?: Snippet;
  } = $props();
</script>

<header
  class="grid grid-cols-3 items-center px-4 py-3 bg-[#252526] border-b border-gray-700 shadow-md z-10"
>
  <!-- Left part -->
  <div class="flex items-center gap-2">
    <a
      href="/"
      class="px-3 py-1 rounded border border-gray-600 text-gray-400 text-xs font-medium hover:bg-[#3c3c3c] transition h-8 flex items-center gap-1.5"
      title="Home"
    >
      <House />
    </a>
  </div>

  <!-- Middle part -->
  <div class="text-center truncate">
    {#if title}
      <h1 class="text-sm font-medium text-gray-200 truncate">{title}</h1>
    {/if}
  </div>

  <!-- Right part -->
  <div class="flex items-center gap-2 mt-5 ml-auto">
    {@render children?.()}
  </div>
</header>
