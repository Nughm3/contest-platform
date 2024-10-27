<script lang="ts">
	import type { PageData } from './$types';
	import { enhance } from '$app/forms';

	interface Props {
		data: PageData;
	}

	let { data }: Props = $props();
</script>

<p class="mb-4 text-3xl font-bold">Admin</p>

<section id="contests">
	<table class="mx-auto">
		<thead>
			<tr>
				<th scope="col">Name</th>
				<th scope="col">Action</th>
			</tr>
		</thead>
		<tbody>
			{#each data.contests as [slug, contest]}
				<tr>
					<th scope="row"><a href={'/contest/' + slug}>{contest.name}</a></th>
					<td>
						<form method="POST" action="?/start" use:enhance>
							<input type="hidden" id="contest" name="contest" value={slug} />
							<button>Start</button>
						</form>
					</td>
				</tr>
			{/each}
		</tbody>
	</table>
</section>
