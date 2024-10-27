import type { Message, ResourceUsage } from '$lib/judge/schema';
import { error } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { createParser } from 'eventsource-parser';
import { db } from '$lib/server/db';
import { contests, submissions, tests } from '$lib/server/db/schema';
import { eq } from 'drizzle-orm';

export const POST: RequestHandler = async ({ fetch, request, params, locals }) => {
	if (!locals.user) error(401);

	const contest = db
		.select({ id: contests.id })
		.from(contests)
		.where(eq(contests.name, params.contest))
		.get();
	if (!contest) error(500, { message: 'contest does not exist' });

	const formData = await request.formData();
	formData.set('contest', params.contest);
	formData.set('task', params.task);

	const response = await fetch('http://localhost:8128', {
		method: 'POST',
		body: formData
	});

	if (!response.ok) return response;

	const reader = response.body!.getReader();
	const decoder = new TextDecoder();

	const codeFile = <File>formData.get("code")!;
	const code = await codeFile.text();

	const parser = createParser({
		onEvent: async (event) => {
			const message: Message = JSON.parse(event.data);
			if (message.type === 'Done') {
				const report = message.report;

				const submission = 
					await db.insert(submissions).values({
						userId: locals.user!.id,
						contestId: contest.id,
						task: parseInt(params.task),
						code,
						language: formData.get('language')!.toString(),
						verdict: report.task,
					})
				;

				const testValues = report.tests.flatMap((tests, subtask) =>
					tests.map((test, index) => ({
						submissionId: Number(submission.lastInsertRowid),
						subtask: subtask + 1,
						index: index + 1,
						runtime: durationToMilliseconds(test.resource_usage),
						memory: test.resource_usage.memory,
						verdict: test.verdict,
					}))
				);

				await db.insert(tests).values(testValues);
			}
		}
	});

	const stream = new ReadableStream({
		async pull(controller) {
			const { done, value } = await reader.read();
			if (done) {
				controller.close();
				return;
			}

			const chunk = decoder.decode(value, { stream: true });
			parser.feed(chunk);
			controller.enqueue(value);
		}
	});

	return new Response(stream, {
		headers: {
			'Content-Type': 'text/event-stream'
		}
	});
};

function durationToMilliseconds(resourceUsage: ResourceUsage): number {
	return resourceUsage['sys-time'].secs * 1000 + resourceUsage['sys-time'].nanos / 1e6 + resourceUsage['user-time'].secs * 1000 + resourceUsage['user-time'].nanos / 1e6;
}
