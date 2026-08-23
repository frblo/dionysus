import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ fetch }) => {
  const response = await fetch("/rooms/api/list");
  const data = await response.json();

  return {
    roomList: data
  };
};
