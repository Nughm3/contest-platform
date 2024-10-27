import { getContest } from '$lib/server/contest/load';
import { db } from '$lib/server/db';
import { submissions } from '$lib/server/db/schema';
import { eq } from 'drizzle-orm';
import { error } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ params, locals }) => {
	const contest = await getContest(params.contest);
	if (!contest) error(404);
	const idx = parseInt(params.task);
	if (!idx || idx === 0) error(404);
	const task = contest.tasks[idx - 1];
	if (!task) error(404);

	const userId = locals.user!.id;
	const previousSubmissions = await db
		.select()
		.from(submissions)
		.where(eq(submissions.userId, userId));

	return {
		name: task.name,
		difficulty: task.difficulty,
		page: task.page,
		languages: Array.from(contest.judge.languages.map((lang) => lang.name)),
		rlimits: contest.judge['resource-limits'],
		submissions: previousSubmissions
	};
};
