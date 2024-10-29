import { getContest } from '$lib/server/contest/load';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ params }) => {
	const contest = await getContest(params.contest);
	return {
		page: contest!.page
	};
};
