import { db } from '$lib/server/db';
import { admins, contests } from '$lib/server/db/schema';
import { eq } from 'drizzle-orm';
import { getContest } from '$lib/server/contest/load';
import type { LayoutServerLoad } from './$types';

export const load: LayoutServerLoad = async ({ locals, params }) => {
	let isAdmin = false;
	if (locals.user) {
		const admin = db.select().from(admins).where(eq(admins.userId, locals.user.id)).get();
		if (admin) isAdmin = true;
	}

	let contest;
	if (params.contest) {
		const contestData = await getContest(params.contest);
		const contestSession = db
			.select()
			.from(contests)
			.where(eq(contests.slug, params.contest))
			.get();
		if (contestData && contestSession) {
			contest = {
				id: contestSession.id,
				slug: params.contest,
				name: contestData.name,
				tasks: Array.from(contestData.tasks.map((task) => task.name)),
				started: contestSession.started,
				duration: contestData.duration
			};
		}
	}

	return {
		username: locals.user?.username,
		isAdmin,
		contest
	};
};
