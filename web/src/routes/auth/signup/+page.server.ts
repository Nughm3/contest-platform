import { lucia } from '$lib/server/auth';
import { db } from '$lib/server/db';
import { user } from '$lib/server/db/schema';
import { fail, redirect } from '@sveltejs/kit';
import { generateIdFromEntropySize } from 'lucia';
import { hash } from '@node-rs/argon2';

import type { Actions } from './$types';

export const actions: Actions = {
	default: async (event) => {
		const formData = await event.request.formData();
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
				message:
					'Invalid username: must be between 3 and 31 characters, only containing alphanumeric characters, hyphens and underscores'
			});
		}
		if (typeof password !== 'string' || password.length < 6 || password.length > 255) {
			return fail(400, {
				message: 'Invalid password: must be at least 6 characters'
			});
		}

		const userId = generateIdFromEntropySize(10); // 16 characters long
		const passwordHash = await hash(password, {
			memoryCost: 19456,
			timeCost: 2,
			outputLen: 32,
			parallelism: 1
		});

		try {
			await db.insert(user).values({
				id: userId,
				username,
				passwordHash
			});
		} catch (e) {
			return fail(409, { message: 'Username taken' });
		}

		const session = await lucia.createSession(userId, {});
		const sessionCookie = lucia.createSessionCookie(session.id);
		event.cookies.set(sessionCookie.name, sessionCookie.value, {
			path: '.',
			...sessionCookie.attributes
		});

		redirect(302, redirectURL.toString());
	}
};
