<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { PageData } from './$types';

	interface Props {
		data: PageData;
		children: Snippet;
	}

	let { data, children }: Props = $props();
</script>

<svelte:head>
	<title>{data.contest!.name} | Contest Platform</title>
</svelte:head>

{@render children()}
