import { sqliteTable, text, integer, real } from 'drizzle-orm/sqlite-core';
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

const verdict = text('verdict', {
		enum: [
			'CompileError',
			'RuntimeError',
			'MemoryLimitExceeded',
			'TimeLimitExceeded',
			'WrongAnswer',
			'Skipped',
			'Accepted'
		]
	}).notNull();

export const submissions = sqliteTable('submissions', {
	id: integer('id').primaryKey(),
	userId: text('user_id')
		.notNull()
		.references(() => users.id, { onDelete: 'cascade' }),
	contestId: integer('contest_id')
		.notNull()
		.references(() => contests.id, { onDelete: 'cascade' }),
	timestamp: integer('timestamp', { mode: 'timestamp' })
		.notNull()
		.default(sql`(unixepoch())`),
	task: integer('task').notNull(),
	code: text('code').notNull(),
	language: text('language').notNull(),
	verdict,
});

export const tests = sqliteTable('tests', {
	id: integer('id').primaryKey(),
	submissionId: integer('submission_id')
		.notNull()
		.references(() => submissions.id, { onDelete: 'cascade' }),
	subtask: integer('subtask').notNull(),
	index: integer('index').notNull(),
	runtime: real('runtime').notNull(),
	memory: integer('memory').notNull(),
	verdict,
});
