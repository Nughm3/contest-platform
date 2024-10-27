import { db } from '$lib/server/db';
import { admins } from '$lib/server/db/schema';
import { eq } from 'drizzle-orm';
import type { LayoutServerLoad } from './$types';
import { getContest } from '$lib/server/contest/load';

export const load: LayoutServerLoad = async ({ locals, params }) => {
	let isAdmin = false;
	if (locals.user) {
		const admin = db.select().from(admins).where(eq(admins.userId, locals.user.id)).get();
		if (admin) isAdmin = true;
	}

	let contest;
	if (params.contest) {
		const contestData = await getContest(params.contest);
		if (contestData) {
			contest = {
				slug: params.contest,
				name: contestData.name,
				tasks: Array.from(contestData.tasks.map((task) => task.name))
			};
		}
	}

	return {
		username: locals.user?.username,
		isAdmin,
		contest
	};
};
