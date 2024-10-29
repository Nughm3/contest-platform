import { lucia } from '$lib/server/auth';
import { db } from '$lib/server/db';
import { admins, users } from '$lib/server/db/schema';
import { fail, redirect } from '@sveltejs/kit';
import { verify } from '@node-rs/argon2';
import { eq } from 'drizzle-orm';
import type { Actions } from './$types';

export const actions: Actions = {
	default: async ({ request, cookies }) => {
		const formData = await request.formData();
		const username = formData.get('username');
		const password = formData.get('password');
		let redirectURL = formData.get('redirect') ?? '/';
		if (redirectURL.slice(0, 5) === '/auth') redirectURL = '/';

		if (typeof username !== 'string') {
			return fail(400, {
				error: 'Invalid username'
			});
		}
		if (typeof password !== 'string') {
			return fail(400, {
				error: 'Invalid password'
			});
		}

		const existingUser = db.select().from(users).where(eq(users.username, username)).get();
		if (!existingUser) {
			return fail(400, {
				error: 'Incorrect username or password'
			});
		}

		const validPassword = await verify(existingUser.passwordHash, password, {
			memoryCost: 19456,
			timeCost: 2,
			outputLen: 32,
			parallelism: 1
		});
		if (!validPassword) {
			return fail(400, { error: 'Incorrect username or password' });
		}

		const session = await lucia.createSession(existingUser.id, {});
		const sessionCookie = lucia.createSessionCookie(session.id);
		cookies.set(sessionCookie.name, sessionCookie.value, {
			path: '.',
			...sessionCookie.attributes
		});

		if (
			redirectURL === '/' &&
			db.select().from(admins).where(eq(admins.userId, existingUser.id)).get()
		)
			redirectURL = '/admin';

		redirect(302, encodeURI(`/${redirectURL.slice(1)}`));
	}
};
