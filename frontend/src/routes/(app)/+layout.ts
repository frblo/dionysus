import type { LayoutLoad } from "./$types";
import { redirect } from "@sveltejs/kit";

export const load: LayoutLoad = async ({ parent, url }) => {
  const { session } = await parent();

  if (!session) {
    const next = encodeURIComponent(url.pathname + url.search);
    throw redirect(302, `/login?next=${next}`);
  }

  return { session };
};
