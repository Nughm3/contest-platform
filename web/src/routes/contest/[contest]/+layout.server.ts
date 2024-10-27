import { getContest } from '$lib/server/contest/load';
import { db } from '$lib/server/db';
import { admins, contests } from '$lib/server/db/schema';
import { eq } from 'drizzle-orm';
import type { LayoutServerLoad } from './$types';
import { error, redirect } from '@sveltejs/kit';

export const load: LayoutServerLoad = async ({ params, locals, url }) => {
	if (!locals.user) {
		const redirectURL = url.pathname + url.search;
		redirect(302, `/auth/login?redirect=${redirectURL}`);
	}

	const started = db.select().from(contests).where(eq(contests.name, params.contest)).get();
	if (!started) {
		const admin = db.select().from(admins).where(eq(admins.userId, locals.user.id)).get();
		if (!admin) error(404);
	}
};
