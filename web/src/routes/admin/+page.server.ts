import type { PageServerLoad, Actions } from './$types';
import { redirect, error, fail } from '@sveltejs/kit';
import { db } from '$lib/server/db';
import { admins, contests } from '$lib/server/db/schema';
import { eq } from 'drizzle-orm';
import { getContests } from '$lib/server/contest/load';

export const load: PageServerLoad = async ({ locals }) => {
	if (!locals.user) redirect(302, '/auth/login?redirect=/admin');
	const admin = db.select().from(admins).where(eq(admins.userId, locals.user.id)).get();
	if (!admin) error(401);

	return {
		contests: Array.from(await getContests())
	};
};

export const actions: Actions = {
	start: async ({ request }) => {
		const formData = await request.formData();
		const name = formData.get('contest')!.toString();
		try {
			await db.insert(contests).values({ name });
		} catch (e) {
			console.log(e);
			return fail(500, { message: 'Failed to create contest' });
		}
	}
};
