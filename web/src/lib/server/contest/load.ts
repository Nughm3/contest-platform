import fs from 'node:fs/promises';
import path from 'node:path';
import { env } from '$env/dynamic/private';
import type { Contest } from './schema';

let contestData: Map<string, Contest> | undefined;

export async function getContests() {
	if (!contestData) {
		contestData = new Map();
		for (const filename of await fs.readdir(env.CONTEST_DATA)) {
			const name = path.parse(filename).name;
			const contents = await fs.readFile(path.join(env.CONTEST_DATA, filename), {
				encoding: 'utf8'
			});
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
