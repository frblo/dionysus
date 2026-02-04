import type { LayoutLoad } from "./$types";
import { redirect } from "@sveltejs/kit";

export const prerender = false;
export const ssr = false;

export const load: LayoutLoad = async ({ fetch, url }) => {
  const r = await fetch("/auth/me", {
    method: "GET",
    credentials: "include",
  });

  if (r.status === 401) {
    const next = encodeURIComponent(url.pathname + url.search);
    throw redirect(302, `/login?next=${next}`);
  }

  if (!r.ok) {
    throw redirect(302, "/login");
  }

  const me = await r.json();
  return { me };
};
