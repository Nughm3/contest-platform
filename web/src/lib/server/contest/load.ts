import fs from 'node:fs/promises';
import path from 'node:path';
import { CONTEST_DATA } from '$env/static/private';
import type { Contest } from './schema';

let contestData: Map<string, Contest> | undefined;

export async function getContests() {
	if (!contestData) {
		contestData = new Map();
		for (const filename of await fs.readdir(CONTEST_DATA)) {
			const name = path.parse(filename).name;
			const contents = await fs.readFile(path.join(CONTEST_DATA, filename), { encoding: 'utf8' });
			let contest: Contest = JSON.parse(contents);
			contest.tasks.map((task) => (task.subtasks = []));
			contestData.set(name, contest);
		}
	}

	return contestData!;
}

export async function getContest(name: string) {
	return await getContests().then((c) => c.get(name));
}
