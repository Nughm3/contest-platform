import { sqliteTable, text, integer } from 'drizzle-orm/sqlite-core';

export const user = sqliteTable('user', {
	id: integer('id').primaryKey(),
	username: text('username').unique().notNull(),
	password_hash: text('password_hash').notNull(),
	email: text('email').notNull()
});
