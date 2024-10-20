import fs from "node:fs/promises";
import path from "node:path";
import { CONTEST_DATA } from "$env/static/private";

let contestData = new Map();
for (const filename of await fs.readdir(CONTEST_DATA)) {
  const name = path.parse(filename).name;
  const contents = await fs.readFile(path.join(CONTEST_DATA, filename), { encoding: 'utf8' });
  const contest = JSON.parse(contents);
  contestData.set(name, contest);
}

export const contests = contestData;
