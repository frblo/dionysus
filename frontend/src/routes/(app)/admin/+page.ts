import type { PageLoad } from "./$types";
import { error } from "@sveltejs/kit";
import type { Users } from "$lib/api/generated/admin/Users";

export const load: PageLoad = async ({ parent, fetch }) => {
  const { session } = await parent();

  if (session?.global_role !== "admin") {
    throw error(403, "Admin access required");
  }

  const response = await fetch("/api/admin/users");
  if (!response.ok) throw error(response.status, "Failed to load users");
  const adminUsers: Users = await response.json();

  return { adminUsers };
};
