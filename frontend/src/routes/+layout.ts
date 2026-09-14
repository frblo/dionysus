import type { LayoutLoad } from "./$types";
import type { Me } from "$lib/api/generated/Me";

export const prerender = false;
export const ssr = false;

export const load: LayoutLoad = async ({ fetch }) => {
  const r = await fetch("/auth/me", {
    method: "GET",
    credentials: "include",
  });

  const session: Me | null = r.ok ? await r.json() : null;
  return { session };
};
