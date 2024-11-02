<script lang="ts">
	import { enhance } from '$app/forms';
	import {
		Heading,
		Button,
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

<Heading tag="h2" class="mb-6">Admin</Heading>

<section id="contests">
	<Table>
		<TableHead>
			<TableHeadCell>Name</TableHeadCell>
			<TableHeadCell>Action</TableHeadCell>
		</TableHead>
		<TableBody>
			{#each data.contests as [slug, contest]}
				<TableBodyRow>
					<TableBodyCell
						><a href={'/contest/' + slug} class="underline">{contest.name}</a></TableBodyCell
					>
					<TableBodyCell>
						<form method="POST" action="?/start" use:enhance>
							<input type="hidden" id="contest" name="contest" value={slug} />
							<Button type="submit">Start</Button>
						</form>
					</TableBodyCell>
				</TableBodyRow>
			{/each}
		</TableBody>
	</Table>
</section>
