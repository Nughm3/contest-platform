<script lang="ts">
	import {
		Heading,
		Table,
		TableBody,
		TableBodyCell,
		TableBodyRow,
		TableHead,
		TableHeadCell
	} from 'flowbite-svelte';
	import type { PageData } from './$types';

	interface Props {
		data: PageData;
	}

	let { data }: Props = $props();
</script>

<Heading tag="h2" class="mb-6">Leaderboard</Heading>

<Table>
	<TableHead>
		<TableHeadCell>Rank</TableHeadCell>
		<TableHeadCell>Player</TableHeadCell>
		<TableHeadCell>Score</TableHeadCell>
	</TableHead>
	<TableBody>
		{#each data.leaderboard as entry, i}
			<TableBodyRow>
				<TableBodyCell><strong>{i + 1}</strong></TableBodyCell>
				<TableBodyCell>{entry.username}</TableBodyCell>
				<TableBodyCell>{entry.totalScore}</TableBodyCell>
			</TableBodyRow>
		{/each}
	</TableBody>
</Table>
