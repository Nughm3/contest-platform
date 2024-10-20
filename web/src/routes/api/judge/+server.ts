import type { Message } from '$lib/judge/schema';
import type { RequestHandler } from './$types';
import { createParser } from 'eventsource-parser';

export const POST: RequestHandler = async ({ fetch, request }) => {
	const formData = await request.formData();

	const response = await fetch('http://localhost:8128', {
		method: 'POST',
		body: formData
	});

	if (!response.ok) return response;

	const reader = response.body!.getReader();
	const decoder = new TextDecoder();

	const parser = createParser({
		onEvent: (event) => {
			const message: Message = JSON.parse(event.data);
			if (message.type === 'Done') {
				const report = message.report;
				// insert into DB
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
