<script lang="ts">
  import "../app.css";
  import favicon from "$lib/assets/favicon.svg";
  import { invalidate } from "$app/navigation";
  import type { LayoutData } from "./$types";
  import { sessionState } from "$lib/state/session.svelte";

  let { data, children }: { data: LayoutData; children: any } = $props();

  $effect(() => {
    sessionState.current = data.session;
  });

  // Only logged in users hold an sse session for auth.
  let loggedIn = $derived(data.session !== null);
  $effect(() => {
    if (!loggedIn) return;

    const eventSource = new EventSource("/auth/sse");

    eventSource.addEventListener("role-changed", () => {
      invalidate("dionysus:session");
    });

    eventSource.addEventListener("resync", () => {
      invalidate("dionysus:session");
    });

    return () => {
      eventSource.close();
    };
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
