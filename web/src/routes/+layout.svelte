<script lang="ts">
	import '../app.css';
	import { page } from '$app/stores';
	import type { LayoutData } from './$types';
	import type { Snippet } from 'svelte';

	interface Props {
		data: LayoutData;
		children: Snippet;
	}

	let { data, children }: Props = $props();

	const redirectURL = $page.url.pathname + $page.url.search;
	const redirectParam =
		redirectURL === '/' || redirectURL.slice(0, 5) === '/auth' ? '' : '?redirect={redirectURL}';
</script>

<main class="mx-auto my-8 w-full max-w-5xl">
	<header>
		<nav class="mb-4 flex justify-between">
			<ul>
				<a href="/">Contest Platform</a>
			</ul>

			<ul>
				{#if data.user}
					<form method="POST" action="/auth/logout"><button>Log out</button></form>
				{:else}
					<a href="/auth/signup{redirectParam}">Sign up</a>
					<a href="/auth/login{redirectParam}">Log in</a>
				{/if}
			</ul>
		</nav>
	</header>

	{@render children()}
</main>
