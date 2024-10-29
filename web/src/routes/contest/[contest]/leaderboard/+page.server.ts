import { db } from '$lib/server/db';
import { users, submissions, contests } from '$lib/server/db/schema';
import { desc, eq, max, sum } from 'drizzle-orm';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ locals, params }) => {
	const contestId = db.select().from(contests).where(eq(contests.slug, params.contest));

	const score = db
		.select({ userId: submissions.userId, maxScore: max(submissions.score).as('maxScore') })
		.from(submissions)
		.where(eq(submissions.contestId, contestId))
		.groupBy(submissions.userId, submissions.task)
		.as('score');

	const leaderboard = await db
		.with(score)
		.select({ userId: users.id, username: users.username, score: sum(score.maxScore) })
		.from(score)
		.leftJoin(users, eq(users.id, score.userId))
		.groupBy(users.id)
		.orderBy(desc(sum(score.maxScore)));

	return { leaderboard };
};
