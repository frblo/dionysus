import type { PageLoad } from './$types';
import { loadRoomList } from '$lib/state/gallery.svelte';

export const load: PageLoad = async ({ fetch }) => {
  await loadRoomList(fetch);
};
