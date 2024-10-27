import { getContests } from '$lib/server/contest/load';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async () => {
	return {
		contests: Array.from(await getContests()).map(([slug, contest]) => [slug, contest.name])
	};
};
