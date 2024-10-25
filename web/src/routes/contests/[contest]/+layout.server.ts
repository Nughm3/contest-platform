import type { LayoutServerLoad } from './$types';
import { redirect } from '@sveltejs/kit';

export const load: LayoutServerLoad = async ({ locals, url }) => {
	if (!locals.user) {
		const redirectURL = url.pathname + url.search;
		redirect(302, `/auth/login?redirect=${redirectURL}`);
	}
};
