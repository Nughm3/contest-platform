import fs from 'node:fs/promises';
import path from 'node:path';
import { CONTEST_DATA } from '$env/static/private';

export interface Contest {
	// name: string;
	// duration: number;
	// "submission-cooldown": number;
	// judge: JudgeConfig;
}

// export interface JudgeConfig {

// }

let contestData: Map<string, Contest> | undefined;

export async function getContests() {
	if (contestData) return contestData;

	contestData = new Map();
	for (const filename of await fs.readdir(CONTEST_DATA)) {
		const name = path.parse(filename).name;
		const contents = await fs.readFile(path.join(CONTEST_DATA, filename), { encoding: 'utf8' });
		const contest = JSON.parse(contents);
		contestData.set(name, contest);
	}
}
