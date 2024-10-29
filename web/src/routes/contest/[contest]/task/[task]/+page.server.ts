import { getContest } from '$lib/server/contest/load';
import { db } from '$lib/server/db';
import { submissions, contests } from '$lib/server/db/schema';
import { eq, and, desc } from 'drizzle-orm';
import { error } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';
import type { Verdict } from '$lib/judge/schema';

// TODO: deduplicate loading, server-side validate submission cooldown

export const load: PageServerLoad = async ({ params, locals }) => {
	const userId = locals.user!.id;

	const contest = db.select().from(contests).where(eq(contests.slug, params.contest)).get();
	if (!contest) error(404);

	const contestData = await getContest(params.contest);
	if (!contestData) error(404);

	const index = parseInt(params.task);
	if (!index || index === 0) error(404);

	const task = contestData.tasks[index - 1];
	if (!task) error(404);

	const previousSubmissions = await db
		.select({
			score: submissions.score,
			verdict: submissions.verdict,
			timestamp: submissions.timestamp
		})
		.from(submissions)
		.where(
			and(
				eq(submissions.userId, userId),
				eq(submissions.contestId, contest.id),
				eq(submissions.task, index)
			)
		)
		.orderBy(desc(submissions.id));

	// TODO: create /submission/[id] route so users can view full data of their own submission
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
		answerSubmission: task.answer != null,
		languages: Array.from(contestData.judge.languages.map((lang) => lang.name)),
		rlimits: contestData.judge['resource-limits'],
		submissionCooldown: contestData['submission-cooldown'],
		submissions: previousSubmissions,
		lastSubmissionTime: previousSubmissions[0]?.timestamp.getTime()
		// tests: previousTests
	};
};

export const actions: Actions = {
	submitAnswer: async ({ params, request, locals }) => {
		const formData = await request.formData();
		const answer = formData.get('answer')?.toString();

		const contest = db.select().from(contests).where(eq(contests.slug, params.contest)).get();

		const contestData = await getContest(params.contest);

		const index = parseInt(params.task);
		const task = contestData!.tasks[index - 1];
		if (!answer || !task.answer) error(400);

		let verdict: Verdict = 'WrongAnswer';
		if (answer.trim() === task.answer.trim()) verdict = 'Accepted';

		db.insert(submissions).values({
			userId: locals.user!.id,
			contestId: contest!.id,
			task: index,
			score: contestData!.scoring['answer-score'],
			verdict
		});

		return { verdict };
	}
};
