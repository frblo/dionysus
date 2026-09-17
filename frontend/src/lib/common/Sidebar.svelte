<script lang="ts">
  import type { Snippet } from "svelte";
  import {
    ExclamationCircle,
    ShieldLock,
    QuestionCircle,
  } from "svelte-bootstrap-icons";
  import { sessionState } from "$lib/state/session.svelte";

  let { children }: { children?: Snippet } = $props();
</script>

<aside
  class="w-12 bg-[#333333] border-r border-gray-700 flex flex-col items-center py-4 gap-4"
>
  <!-- Top buttons -->
  {@render children?.()}

  <!-- Bottom buttons -->
  <div class="mt-auto flex flex-col items-center gap-4">
    {#if sessionState.current?.global_role === "admin"}
      <a href="/admin">
        <button
          class="p-2 text-gray-400 hover:text-white transition-colors"
          title="Admin"
        >
          <ShieldLock />
        </button>
      </a>
    {/if}
    <a href="/help" target="_blank" rel="noreferrer">
      <button
        class="p-2 text-gray-400 hover:text-white transition-colors"
        title="Help"
      >
        <QuestionCircle />
      </button>
    </a>
    <a
      href="https://github.com/frblo/dionysus/issues"
      target="_blank"
      rel="noreferrer"
    >
      <button
        class="p-2 text-gray-400 hover:text-white transition-colors"
        title="Report issue"
      >
        <ExclamationCircle />
      </button>
    </a>
  </div>
</aside>
