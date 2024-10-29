import { db } from '$lib/server/db';
import { admins, contests } from '$lib/server/db/schema';
import { eq } from 'drizzle-orm';
import { error, redirect } from '@sveltejs/kit';
import type { LayoutServerLoad } from './$types';

export const load: LayoutServerLoad = async ({ params, locals, url }) => {
	if (!locals.user) {
		const redirectURL = url.pathname + url.search;
		redirect(302, `/auth/login?redirect=${redirectURL}`);
	}

	const contest = db.select().from(contests).where(eq(contests.slug, params.contest)).get();
	if (!contest) {
		const admin = db.select().from(admins).where(eq(admins.userId, locals.user.id)).get();
		if (!admin) error(404);
	}
};
