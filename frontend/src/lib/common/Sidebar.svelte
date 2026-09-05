<script lang="ts">
  import type { Snippet } from "svelte";
  import { ExclamationCircle, QuestionCircle } from "svelte-bootstrap-icons";

  let { children }: { children?: Snippet } = $props();
</script>

<aside
  class="w-12 bg-[#333333] border-r border-gray-700 flex flex-col items-center py-4 gap-4"
>
  <!-- Top buttons -->
  {@render children?.()}

  <!-- Bottom buttons -->
  <div class="mt-auto flex flex-col items-center gap-4">
    <a href="/help" target="_blank" rel="noreferrer">
      <button
        class="p-2 text-gray-400 hover:text-white transition-colors"
        title="Report issue"
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
