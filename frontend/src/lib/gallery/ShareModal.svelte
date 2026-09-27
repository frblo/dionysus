<script lang="ts">
  import Modal from "$lib/common/Modal.svelte";
  import { GalleryModals, galleryState } from "$lib/state/gallery.svelte";
  import type { RoomMember } from "$lib/api/generated/RoomMember";
  import type { RoomRole } from "$lib/api/generated/RoomRole";
  import type { Candidate } from "$lib/api/generated/rooms/members/Candidate";
  import { debounce } from "$lib/utils/debounce";

  const ROLES: RoomRole[] = ["viewer", "editor", "owner"];

  let members = $state<RoomMember[]>([]);
  let searchInput = $state("");
  let searchResults = $state<Candidate[]>([]);
  let selectedRole = $state<Record<string, RoomRole>>({});
  let errorMessage = $state("");

  function closeShareModal() {
    galleryState.modalOpen = GalleryModals.None;
    searchInput = "";
    searchResults = [];
    errorMessage = "";
  }

  async function loadMembers() {
    const response = await fetch(
      `/api/rooms/members/${galleryState.targetedId}`,
      { credentials: "include" },
    );
    if (response.ok) {
      members = await response.json();
    }
  }

  const runSearch = debounce(async (q: string) => {
    if (q.trim().length < 2) {
      searchResults = [];
      return;
    }
    const response = await fetch(
      `/api/rooms/members_search/${galleryState.targetedId}?q=${encodeURIComponent(q)}`,
      { credentials: "include" },
    );
    searchResults = response.ok ? await response.json() : [];
  }, 300);

  function onSearchInput() {
    runSearch(searchInput);
  }

  $effect(() => {
    if (galleryState.modalOpen === GalleryModals.Share) {
      loadMembers();
    }
  });

  async function addMember(userId: string) {
    errorMessage = "";
    const role = selectedRole[userId] ?? "viewer";

    const response = await fetch(
      `/api/rooms/members/${galleryState.targetedId}`,
      {
        method: "POST",
        credentials: "include",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ user_id: userId, role }),
      },
    );

    if (response.ok) {
      searchResults = searchResults.filter((c) => c.id !== userId);
      await loadMembers();
    } else {
      errorMessage = "Failed to add member.";
    }
  }

  async function removeMember(userId: string) {
    errorMessage = "";

    const response = await fetch(
      `/api/rooms/members/${galleryState.targetedId}/${userId}`,
      { method: "DELETE", credentials: "include" },
    );

    if (response.ok) {
      await loadMembers();
    } else {
      errorMessage = "Failed to remove member.";
    }
  }
</script>

{#if galleryState.modalOpen === GalleryModals.Share}
  <Modal title="Share screenplay" closeModal={closeShareModal}>
    {#if errorMessage}
      <p class="text-red-400 text-xs mb-3">{errorMessage}</p>
    {/if}

    <div class="mb-4">
      <h3 class="text-xs uppercase text-gray-500 mb-2">Members</h3>
      {#if members.length === 0}
        <p class="text-xs text-gray-500">No members yet.</p>
      {/if}
      <ul class="space-y-1">
        {#each members as member (member.user_id)}
          <li class="flex items-center justify-between text-xs text-gray-300">
            <span class="font-mono truncate">{member.display_name}</span>
            <span class="flex items-center gap-2 shrink-0">
              <span class="uppercase text-gray-500">{member.role}</span>
              <button
                class="text-red-400 hover:text-red-300"
                onclick={() => removeMember(member.user_id)}
              >
                Remove
              </button>
            </span>
          </li>
        {/each}
      </ul>
    </div>

    <div>
      <h3 class="text-xs uppercase text-gray-500 mb-2">Add someone</h3>
      <input
        type="text"
        placeholder="Search by name (2+ characters)"
        bind:value={searchInput}
        oninput={onSearchInput}
        class="w-full px-3 py-2 rounded border border-gray-600 bg-[#1e1e1e] text-gray-200 text-sm placeholder-gray-500 focus:outline-none focus:border-gray-400 mb-2"
      />
      <p class="text-[10px] text-gray-500 mb-2">
        Matched by name only - double-check the join date before adding someone
        you don't recognize.
      </p>
      <ul class="space-y-1 max-h-40 overflow-auto">
        {#each searchResults as candidate (candidate.id)}
          <li
            class="flex items-center justify-between text-xs text-gray-300 gap-2"
          >
            <span class="font-mono truncate">
              {candidate.display_name}
              <span class="text-gray-500">
                · joined {new Date(candidate.created_at).toLocaleDateString()}
              </span>
            </span>
            <span class="flex items-center gap-1 shrink-0">
              <select
                class="px-1 py-0.5 rounded border border-gray-600 bg-[#252526] text-gray-200 text-xs"
                bind:value={selectedRole[candidate.id]}
              >
                {#each ROLES as role}
                  <option value={role}>{role}</option>
                {/each}
              </select>
              <button
                class="px-2 py-0.5 rounded bg-blue-600/80 hover:bg-blue-500 text-white text-xs"
                onclick={() => addMember(candidate.id)}
              >
                Add
              </button>
            </span>
          </li>
        {/each}
      </ul>
    </div>
  </Modal>
{/if}
