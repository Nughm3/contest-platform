import { lucia } from '$lib/server/auth';
import { db } from '$lib/server/db';
import { user } from '$lib/server/db/schema';
import { fail, redirect } from '@sveltejs/kit';
import { verify } from '@node-rs/argon2';

import type { Actions } from './$types';
import { eq } from 'drizzle-orm';

export const actions: Actions = {
	default: async (event) => {
		const formData = await event.request.formData();
		const username = formData.get('username');
		const password = formData.get('password');
		let redirectURL = formData.get('redirect') ?? '/';
		if (redirectURL.slice(0, 5) === '/auth') redirectURL = '/';

		if (typeof username !== 'string') {
			return fail(400, {
				message: 'Invalid username'
			});
		}
		if (typeof password !== 'string') {
			return fail(400, {
				message: 'Invalid password'
			});
		}

		const existingUser = db.select().from(user).where(eq(user.username, username)).get();

		if (!existingUser) {
			return fail(400, {
				message: 'Incorrect username or password'
			});
		}

		const validPassword = await verify(existingUser.passwordHash, password, {
			memoryCost: 19456,
			timeCost: 2,
			outputLen: 32,
			parallelism: 1
		});
		if (!validPassword) {
			return fail(400, {
				message: 'Incorrect username or password'
			});
		}

		const session = await lucia.createSession(existingUser.id, {});
		const sessionCookie = lucia.createSessionCookie(session.id);
		event.cookies.set(sessionCookie.name, sessionCookie.value, {
			path: '.',
			...sessionCookie.attributes
		});

		redirect(302, encodeURI(`/${redirectURL.slice(1)}`));
	}
};
