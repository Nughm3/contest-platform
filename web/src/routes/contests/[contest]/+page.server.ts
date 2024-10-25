import type { PageServerLoad } from './$types';
import { getContest } from '$lib/server/contest/load';
import { error } from '@sveltejs/kit';

export const load: PageServerLoad = async ({ params }) => {
	const contest = await getContest(params.contest);
	if (!contest) error(404);

	return {
		name: contest.name,
		page: contest.page,
		tasks: Array.from(contest.tasks.map((task) => task.name))
	};
};
