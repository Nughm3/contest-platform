import { sqliteTable, text, integer } from 'drizzle-orm/sqlite-core';
import { sql } from 'drizzle-orm';

export const user = sqliteTable('user', {
	id: text('id').primaryKey(),
	username: text('username').notNull().unique(),
	passwordHash: text('password_hash').notNull()
});

export const session = sqliteTable('session', {
	id: text('id').primaryKey(),
	userId: text('user_id')
		.notNull()
		.references(() => user.id, { onDelete: 'cascade' }),
	expiresAt: integer('expires_at').notNull()
});

export const submission = sqliteTable('submission', {
	id: text('id').primaryKey(),
	userId: text('user_id')
		.notNull()
		.references(() => user.id, { onDelete: 'cascade' }),
	timestamp: integer('timestamp', { mode: 'timestamp' })
		.notNull()
		.default(sql`(unixepoch())`),
	contest: text('contest').notNull(),
	task: integer('task').notNull(),
	code: text('code').notNull(),
	language: text('language').notNull()
});

export const tests = sqliteTable('tests', {
	id: text('id').primaryKey(),
	submissionId: text('submission_id')
		.notNull()
		.references(() => submission.id, { onDelete: 'cascade' }),
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
