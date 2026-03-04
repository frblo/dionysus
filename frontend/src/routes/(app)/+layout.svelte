<script lang="ts">
  import type { LayoutData } from "./$types";
  let { data, children }: { data: LayoutData; children: any } = $props();
</script>

{#if data.me}
  {@render children()}
{:else}
  <!-- render nothing or a minimal loader -->
  <p>Checking session…</p>
{/if}
