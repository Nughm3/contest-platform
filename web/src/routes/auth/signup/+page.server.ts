import { lucia } from '$lib/server/auth';
import { db } from '$lib/server/db';
import { users } from '$lib/server/db/schema';
import { fail, redirect } from '@sveltejs/kit';
import { eq } from 'drizzle-orm';
import { generateIdFromEntropySize } from 'lucia';
import { hash } from '@node-rs/argon2';

import type { Actions } from './$types';

function usernameAvailable(username: string): boolean {
	const user = db.select().from(users).where(eq(users.username, username)).get();
	return user === undefined;
}

export const actions: Actions = {
	default: async ({ request, cookies }) => {
		const formData = await request.formData();
		const username = formData.get('username');
		const password = formData.get('password');
		let redirectURL = formData.get('redirect') ?? '/';
		if (redirectURL.slice(0, 5) === '/auth') redirectURL = '/';

		if (
			typeof username !== 'string' ||
			username.length < 3 ||
			username.length > 31 ||
			!/^[a-zA-Z0-9_-]+$/.test(username)
		) {
			return fail(400, {
				username,
				error:
					'Invalid username: must be between 3 and 31 characters long, only containing alphanumeric characters, hyphens and underscores'
			});
		}
		if (typeof password !== 'string' || password.length < 6 || password.length > 255) {
			return fail(400, {
				username,
				error: 'Invalid password: must be between 6 and 255 characters long'
			});
		}

		if (!usernameAvailable(username)) {
			return fail(409, {
				username,
				error: 'Username taken'
			});
		}

		const userId = generateIdFromEntropySize(10); // 16 characters long
		const passwordHash = await hash(password, {
			memoryCost: 19456,
			timeCost: 2,
			outputLen: 32,
			parallelism: 1
		});

		await db.insert(users).values({
			id: userId,
			username,
			passwordHash
		});

		const session = await lucia.createSession(userId, {});
		const sessionCookie = lucia.createSessionCookie(session.id);
		cookies.set(sessionCookie.name, sessionCookie.value, {
			path: '.',
			...sessionCookie.attributes
		});

		redirect(302, redirectURL.toString());
	}
};
