<script lang="ts">
  export let data: { me?: unknown };
</script>

{#if data.me}
  <slot />
{:else}
  <!-- render nothing or a minimal loader -->
  <p>Checking session…</p>
{/if}
