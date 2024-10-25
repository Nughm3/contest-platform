import fs from 'node:fs/promises';
import path from 'node:path';
import { CONTEST_DATA } from '$env/static/private';
import type { Contest } from './schema';

let contestData: Map<string, Contest> | undefined;

async function initializeContests() {
	if (!contestData) {
		contestData = new Map();
		for (const filename of await fs.readdir(CONTEST_DATA)) {
			const name = path.parse(filename).name;
			const contents = await fs.readFile(path.join(CONTEST_DATA, filename), { encoding: 'utf8' });
			let contest: Contest = JSON.parse(contents);
			contest.tasks.map(task => task.subtasks = []);
			contestData.set(name, contest);
		}
	}

	return contestData!;
}

export async function getContest(name: string) {
	return await initializeContests().then(c => c.get(name));
}

export async function listContests() {
	const contests = await initializeContests();
	return Array.from(contests.keys());
}
