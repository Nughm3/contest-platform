import { getContest } from '$lib/server/contest/load';
import { fail } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ params }) => {
	const contest = await getContest(params.name);
	if (!contest) throw fail(404);

	const task = contest.tasks[parseInt(params.task)];
	if (!task) throw fail(404);

	return {
		name: task.name,
		difficulty: task.difficulty,
		page: task.page
	};
};
