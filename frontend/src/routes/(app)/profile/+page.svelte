<script lang="ts">
  import Header from "$lib/common/Header.svelte";
  import Sidebar from "$lib/common/Sidebar.svelte";
  import type { PageData } from "./$types";

  let { data }: { data: PageData } = $props();

  let displayName = $state(data.profile.display_name);
  const email = data.profile.email;
  const providers = data.profile.providers;

  let pending = $state(false);
  let errorMessage = $state("");
  let saved = $state(false);

  async function save() {
    pending = true;
    errorMessage = "";
    saved = false;

    const response = await fetch("/auth/profile", {
      method: "PATCH",
      credentials: "include",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ display_name: displayName }),
    });

    if (response.ok) {
      const profile = await response.json();
      displayName = profile.display_name;
      saved = true;
    } else {
      errorMessage = "Failed to update display name. Please try again.";
    }

    pending = false;
  }
</script>

<Header title="Profile"></Header>

<div class="flex flex-1 h-[calc(100vh-64px)] overflow-hidden">
  <Sidebar />

  <main class="flex-1 overflow-auto bg-[#1e1e1e] p-6">
    <div class="max-w-md">
      {#if errorMessage}
        <p class="text-red-400 text-xs mb-4">{errorMessage}</p>
      {/if}

      <div class="mb-6">
        <label
          for="display-name"
          class="block text-xs uppercase text-gray-500 mb-2"
        >
          Display name
        </label>
        <div class="flex gap-2">
          <input
            id="display-name"
            type="text"
            bind:value={displayName}
            disabled={pending}
            class="flex-1 px-2 py-1 rounded border border-gray-600 bg-[#252526] text-gray-200 text-sm"
          />
          <button
            onclick={save}
            disabled={pending}
            class="px-3 py-1 rounded border border-gray-600 text-gray-200 text-sm hover:bg-[#333333] transition-colors"
          >
            Save
          </button>
        </div>
        {#if saved}
          <p class="text-xs text-gray-500 mt-1">Saved.</p>
        {/if}
      </div>

      <div class="mb-6">
        <span class="block text-xs uppercase text-gray-500 mb-2">Email</span>
        <p class="text-sm text-gray-300 font-mono">{email ?? "Not set"}</p>
      </div>

      <div>
        <span class="block text-xs uppercase text-gray-500 mb-2"
          >Linked providers</span
        >
        <ul class="text-sm text-gray-300 font-mono">
          {#each providers as provider}
            <li>{provider}</li>
          {/each}
        </ul>
      </div>
    </div>
  </main>
</div>
