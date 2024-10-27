import type { PageServerLoad } from './$types';
import { getContest } from '$lib/server/contest/load';

export const load: PageServerLoad = async ({ params }) => {
	const contest = await getContest(params.contest);
	return {
		page: contest!.page
	};
};
