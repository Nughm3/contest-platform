import { getContest } from '$lib/server/contest/load';
import { db } from '$lib/server/db';
import { submissions, contests, tests } from '$lib/server/db/schema';
import { eq, and, desc } from 'drizzle-orm';
import { error } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ params, locals }) => {
	const contest = await getContest(params.contest);
	if (!contest) error(404);
	const index = parseInt(params.task);
	if (!index || index === 0) error(404);
	const task = contest.tasks[index - 1];
	if (!task) error(404);

	const userId = locals.user!.id;
	const contestId = db.select().from(contests).where(eq(contests.name, params.contest)).get()!.id;
	const previousSubmissions = await db
		.select({ verdict: submissions.verdict })
		.from(submissions)
		.where(
			and(
				eq(submissions.userId, userId),
				eq(submissions.contestId, contestId),
				eq(submissions.task, index)
			)
		)
		.orderBy(desc(submissions.id));

	// const previousTests = await db
	// 	.select({
	// 		submissionId: submissions.id,
	// 		subtask: tests.subtask,
	// 		index: tests.index,
	// 		runtime: tests.runtime,
	// 		memory: tests.memory,
	// 		verdict: tests.verdict
	// 	})
	// 	.from(submissions)
	// 	.leftJoin(tests, eq(tests.submissionId, submissions.id))
	// 	.where(
	// 		and(
	// 			eq(submissions.userId, userId),
	// 			eq(submissions.contestId, contestId),
	// 			eq(submissions.task, index)
	// 		)
	// 	)
	// 	.orderBy(desc(submissions.id), tests.subtask, tests.index);

	return {
		name: task.name,
		difficulty: task.difficulty,
		page: task.page,
		languages: Array.from(contest.judge.languages.map((lang) => lang.name)),
		rlimits: contest.judge['resource-limits'],
		submissions: previousSubmissions,
		// tests: previousTests
	};
};
