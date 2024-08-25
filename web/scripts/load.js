#!/usr/bin/env node

import fs from 'node:fs/promises';
import markdownit from 'markdown-it';
import mk from '@vscode/markdown-it-katex';
import path from 'node:path';
import TOML from 'smol-toml';

const md = markdownit().use(mk.default);
const input = process.argv[2];

function renderMarkdown(src) {
	src = src.trim();
	let frontmatter = undefined;

	if (src.slice(0, 3) === '+++') {
		const end = src.indexOf('+++', 3);
		if (end === -1) throw new Error('TOML frontmatter parse error');
		frontmatter = TOML.parse(src.slice(3, end));
		src = src.slice(end + 3).trim();
	}

	const html = md.render(src);
	return { frontmatter, html };
}

const contestPage = await fs.readFile(path.join(input, 'contest.md'), { encoding: 'utf8' });
const { frontmatter, html } = renderMarkdown(contestPage);
let contest = {
	config: frontmatter,
	page: html,
	tasks: []
};

const taskEntries = await fs.readdir(input, { withFileTypes: true });
for (const task of taskEntries.sort().filter((e) => e.isDirectory())) {
	const taskPage = await fs.readFile(path.join(input, task.name, 'task.md'), { encoding: 'utf8' });

	const { frontmatter, html } = renderMarkdown(taskPage);
	if (!frontmatter) throw new Error(`task ${task.name} has no TOML front matter`);

	let subtasks = [];

	if (!frontmatter.answer) {
		const subtaskEntries = await fs.readdir(path.join(task.parentPath, task.name), {
			withFileTypes: true
		});

		for (const subtask of subtaskEntries.sort().filter((e) => e.isDirectory())) {
			let tests = [];
			for (let i = 1; ; i++) {
				const basePath = path.join(subtask.parentPath, subtask.name);
				try {
					tests.push({
						input: await fs.readFile(path.join(basePath, `${i}.out`), { encoding: 'utf8' }),
						output: await fs.readFile(path.join(basePath, `${i}.in`), { encoding: 'utf8' })
					});
				} catch {
					break;
				}
			}

			subtasks.push(tests);
		}
	}

	contest.tasks.push({
		config: frontmatter,
		page: html,
		subtasks
	});
}

const output = path.parse(input).name + '.contest.json';
await fs.writeFile(output, JSON.stringify(contest));
console.log(`output written to ${output}`);
