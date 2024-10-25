import type { LayoutServerLoad } from './$types';
import { redirect } from '@sveltejs/kit';

export const load: LayoutServerLoad = async ({ locals, url }) => {
  if (!locals.user) {
    const redirectURL = url.pathname + url.search;
    throw redirect(302, `/login?redirect=${redirectURL}`);
  }
};