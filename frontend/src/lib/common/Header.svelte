<script lang="ts">
  import type { Snippet } from "svelte";
  import { House } from "svelte-bootstrap-icons";

  let {
    title = "",
    children,
    leftChildren,
  }: {
    title?: string;
    children?: Snippet;
    leftChildren?: Snippet;
  } = $props();
</script>

<header
  class="grid grid-cols-3 items-center px-4 py-3 bg-[#252526] border-b border-gray-700 shadow-md z-10"
>
  <!-- Left part -->
  <div class="flex items-center gap-2">
    <a
      href="/"
      class="px-3 py-1 rounded bg-[#f77f00] text-gray-100 text-xs font-medium hover:bg-[#fcbf49] transition h-8 flex items-center gap-1.5"
      title="Home"
    >
      <House />
    </a>
    {#if leftChildren}
      <div class="w-px h-6 bg-gray-700 mx-1" aria-hidden="true"></div>
    {/if}
    {@render leftChildren?.()}
  </div>

  <!-- Middle part -->
  <div class="text-center truncate">
    {#if title}
      <h1 class="text-sm font-medium text-gray-200 truncate">{title}</h1>
    {/if}
  </div>

  <!-- Right part -->
  <div class="flex items-center gap-2 ml-auto">
    {@render children?.()}
  </div>
</header>
