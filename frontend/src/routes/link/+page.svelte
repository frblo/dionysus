<script lang="ts">
  import { BoxArrowInRight } from "svelte-bootstrap-icons";
  import { page } from "$app/state";
  import type { PendingLinkInfo } from "$lib/api/generated/PendingLinkInfo";

  let token = page.url.searchParams.get("token") ?? "";

  let error = $state("");
  let loading = $state(true);
  let info = $state<PendingLinkInfo | null>(null);

  async function loadLinkInfo() {
    if (!token) {
      error = "Missing link token";
      loading = false;
      return;
    }

    try {
      const res = await fetch(`/auth/link/${token}`);
      if (!res.ok) throw new Error(`HTTP ${res.status}: ${res.statusText}`);
      info = (await res.json()) as PendingLinkInfo;
    } catch (err) {
      error = err instanceof Error ? err.message : "Unknown error";
    } finally {
      loading = false;
    }
  }

  function confirm(provider: string) {
    window.location.href = `/auth/login?provider=${provider}&link_token=${token}`;
  }

  loadLinkInfo();
</script>

<svelte:head>
  <title>Confirm it's you - Dionysus</title>
</svelte:head>

<main
  class="flex flex-col items-center justify-center min-h-screen bg-[#1e1e1e] text-gray-100 p-4"
>
  <div class="text-center mb-12 max-w-md">
    <h1 class="title-text">confirm it's you</h1>
    {#if info}
      <p class="subtitle">
        An account already exists for <strong>{info.masked_email}</strong>. Log
        in below to confirm it's you.
      </p>
    {/if}
  </div>

  {#if loading}
    <p>Loading...</p>
  {:else if error}
    <p>Error: {error}</p>
  {:else if info}
    {#each info.target_providers as id}
      <button onclick={() => confirm(id)} class="login-button">
        <BoxArrowInRight />
        {id}
      </button>
    {/each}
  {/if}
</main>

<style>
  .title-text {
    font-family: "Courier New", Courier, monospace;
    font-size: clamp(1.5rem, 8vw, 40px);
    font-weight: bold;
    letter-spacing: -0.05em;
  }

  .subtitle {
    font-family: "Courier New", Courier, monospace;
    color: #a0a0a0;
    margin-top: 1rem;
  }

  .login-button {
    font-family: "Courier New", Courier, monospace;
    font-size: 18px;
    padding: 12px 32px;
    border: 2px solid #4a4a4a;
    border-radius: 4px;
    color: #a0a0a0;
    transition: all 0.2s ease;
    text-decoration: none;
    letter-spacing: 2px;
    width: 240px;
    display: inline-flex;
    align-items: center;
    gap: 10px;
  }

  .login-button:hover {
    background-color: #333333;
    color: #ffffff;
    border-color: #a0a0a0;
  }
</style>
