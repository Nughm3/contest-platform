import { getContests } from '$lib/server/contest/load';
import {db} from '$lib/server/db';
import { contests } from '$lib/server/db/schema';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async () => {
	const contestData = await getContests();
	const contestSessions = await db.select().from(contests);

	return {
		contests: contestSessions.map(contest => {
            const data = contestData.get(contest.slug)!;
            return {
            	name: data.name,
            	slug: contest.slug,
            	started: contest.started,
            }
        })
	};
};
