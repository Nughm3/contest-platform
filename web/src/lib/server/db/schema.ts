import { sqliteTable, text, integer } from 'drizzle-orm/sqlite-core';
import { sql } from 'drizzle-orm';

export const users = sqliteTable('users', {
	id: text('id').primaryKey(),
	username: text('username').notNull().unique(),
	passwordHash: text('password_hash').notNull()
});

export const sessions = sqliteTable('sessions', {
	id: text('id').primaryKey(),
	userId: text('user_id')
		.notNull()
		.references(() => users.id, { onDelete: 'cascade' }),
	expiresAt: integer('expires_at').notNull()
});

export const admins = sqliteTable('admins', {
	userId: text('user_id')
		.unique()
		.notNull()
		.references(() => users.id, { onDelete: 'cascade' })
});

export const contests = sqliteTable('contests', {
	id: integer('id').primaryKey(),
	name: text('name').notNull(),
	started: integer('started', { mode: 'timestamp' })
		.notNull()
		.default(sql`(unixepoch())`)
});

export const submissions = sqliteTable('submissions', {
	id: integer('id').primaryKey(),
	userId: text('user_id')
		.notNull()
		.references(() => users.id, { onDelete: 'cascade' }),
	contestId: text('contest_id')
		.notNull()
		.references(() => contests.id, { onDelete: 'cascade' }),
	timestamp: integer('timestamp', { mode: 'timestamp' })
		.notNull()
		.default(sql`(unixepoch())`),
	contest: text('contest').notNull(),
	task: integer('task').notNull(),
	code: text('code').notNull(),
	language: text('language').notNull()
});

export const tests = sqliteTable('tests', {
	id: integer('id').primaryKey(),
	submissionId: text('submission_id')
		.notNull()
		.references(() => submissions.id, { onDelete: 'cascade' }),
	output: text('output').notNull(),
	runtime: integer('runtime').notNull(),
	memory: integer('memory').notNull(),
	verdict: text('verdict', {
		enum: [
			'CompileError',
			'RuntimeError',
			'MemoryLimitExceeded',
			'TimeLimitExceeded',
			'WrongAnswer',
			'Skipped',
			'Accepted'
		]
	})
});
