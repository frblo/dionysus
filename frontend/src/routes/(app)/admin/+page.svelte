<script lang="ts">
  import Header from "$lib/common/Header.svelte";
  import Sidebar from "$lib/common/Sidebar.svelte";
  import type { GlobalRole } from "$lib/api/generated/GlobalRole";
  import type { PageData } from "./$types";

  let { data }: { data: PageData } = $props();

  let users = $state(data.adminUsers.users);
  const roleManagementEnabled = data.adminUsers.role_management_enabled;

  const ROLES: GlobalRole[] = ["guest", "user", "admin"];

  function roleLabel(role: GlobalRole): string {
    return role.charAt(0).toUpperCase() + role.slice(1);
  }

  let pendingId = $state<string | null>(null);
  let errorMessage = $state("");

  async function changeRole(userId: string, role: GlobalRole) {
    pendingId = userId;
    errorMessage = "";

    const response = await fetch(`/api/admin/users/${userId}/role`, {
      method: "PATCH",
      credentials: "include",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ role }),
    });

    if (response.ok) {
      users = users.map((u) =>
        u.id === userId ? { ...u, global_role: role } : u,
      );
    } else {
      errorMessage = "Failed to update role. Please try again.";
    }

    pendingId = null;
  }
</script>

<Header title="Admin"></Header>

<div class="flex flex-1 h-[calc(100vh-64px)] overflow-hidden">
  <Sidebar />

  <main class="flex-1 overflow-auto bg-[#1e1e1e] p-6">
    {#if errorMessage}
      <p class="text-red-400 text-xs mb-4">{errorMessage}</p>
    {/if}

    {#if !roleManagementEnabled}
      <p class="text-gray-400 text-xs mb-4">
        Global roles are managed by an external system and can't be changed
        here.
      </p>
    {/if}

    <table class="w-full text-sm text-left text-gray-300">
      <thead class="text-xs uppercase text-gray-500 border-b border-gray-700">
        <tr>
          <th class="py-2 pr-4 font-medium">Name</th>
          <th class="py-2 pr-4 font-medium">Role</th>
          <th class="py-2 pr-4 font-medium">Joined</th>
        </tr>
      </thead>
      <tbody>
        {#each users as user (user.id)}
          <tr class="border-b border-gray-800">
            <td class="py-2 pr-4 font-mono">{user.display_name}</td>
            <td class="py-2 pr-4">
              {#if roleManagementEnabled}
                <select
                  class="px-2 py-1 rounded border border-gray-600 bg-[#252526] text-gray-200 text-xs"
                  value={user.global_role}
                  disabled={pendingId === user.id}
                  onchange={(e) =>
                    changeRole(user.id, e.currentTarget.value as GlobalRole)}
                >
                  {#each ROLES as role}
                    <option value={role}>{roleLabel(role)}</option>
                  {/each}
                </select>
              {:else}
                <span class="text-xs text-gray-400"
                  >{roleLabel(user.global_role)}</span
                >
              {/if}
            </td>
            <td class="py-2 pr-4 text-gray-500">
              {new Date(user.created_at).toLocaleDateString()}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </main>
</div>
