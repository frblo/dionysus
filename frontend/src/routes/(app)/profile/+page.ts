import type { PageLoad } from "./$types";
import { error } from "@sveltejs/kit";
import type { Profile } from "$lib/api/generated/Profile";

export const load: PageLoad = async ({ fetch }) => {
  const response = await fetch("/auth/profile");
  if (!response.ok) throw error(response.status, "Failed to load profile");
  const profile: Profile = await response.json();

  return { profile };
};
