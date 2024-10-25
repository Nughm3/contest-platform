import type { PageServerLoad } from './$types';
import { listContests } from '$lib/server/contest/load';

export const load: PageServerLoad = async () => {
	return {
		contests: await listContests()
	};
};
