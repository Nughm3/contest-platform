<script lang="ts">
	import type { PageData } from './$types';
	import {
		Heading,
		Table,
		TableBody,
		TableBodyCell,
		TableBodyRow,
		TableHead,
		TableHeadCell
	} from 'flowbite-svelte';

	interface Props {
		data: PageData;
	}

	let { data }: Props = $props();
</script>

<div class="prose mb-6">
	{#if data.username}
		<Heading tag="h2" class="mb-6">Welcome back, {data.username}!</Heading>
		<p>Show off your competitive programming skills by joining one of the below contests!</p>
	{:else}
		<Heading tag="h2" class="mb-6">Welcome! 👋</Heading>
		<p>
			<a href="/auth/signup">Sign up</a> or <a href="/auth/login">log in</a> to participate in a contest!
		</p>
	{/if}
</div>

<section id="contests">
	<Heading tag="h3" class="mb-6">Contests</Heading>
	{#if data.contests.length !== 0}
		<Table>
			<TableHead>
				<TableHeadCell>Contest</TableHeadCell>
				<TableHeadCell>Started</TableHeadCell>
				<TableHeadCell>Status</TableHeadCell>
			</TableHead>
			<TableBody>
				{#each data.contests as contest}
					<TableBodyRow>
						<TableBodyCell>
							<a href={'/contest/' + contest.slug} class="underline">{contest.name}</a>
						</TableBodyCell>
						<TableBodyCell>{contest.started.toLocaleString()}</TableBodyCell>
						<TableBodyCell>
							{#if contest.active}
								<span class="text-green-700">Active</span>
							{:else}
								<span class="text-red-700">Ended</span>
							{/if}
						</TableBodyCell>
					</TableBodyRow>
				{/each}
			</TableBody>
		</Table>
	{:else}
		<p class="text-sm"><em>No contests yet</em></p>
	{/if}
</section>
