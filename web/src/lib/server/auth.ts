import { Lucia } from 'lucia';
import { dev } from '$app/environment';
import { db } from './db';
import { session, user } from './db/schema';
import { DrizzleSQLiteAdapter } from '@lucia-auth/adapter-drizzle';

const adapter = new DrizzleSQLiteAdapter(db, session, user);

export const lucia = new Lucia(adapter, {
	sessionCookie: {
		attributes: {
			secure: !dev
		}
	},
	getUserAttributes: (attributes) => {
		return {
			username: attributes.username
		};
	}
});

declare module 'lucia' {
	interface Register {
		Lucia: typeof lucia;
		DatabaseUserAttributes: DatabaseUserAttributes;
	}
}

interface DatabaseUserAttributes {
	username: string;
}
