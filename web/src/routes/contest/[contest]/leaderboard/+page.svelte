<script lang="ts">
	import { Heading } from 'flowbite-svelte';
	import type { PageData } from './$types';

	interface Props {
		data: PageData;
	}

	let { data }: Props = $props();
</script>

<Heading tag="h2" class="mb-6">Leaderboard</Heading>

<ol>
	{#each data.leaderboard as entry}
		<li>{entry.username}: {entry.score}</li>
	{/each}
</ol>
