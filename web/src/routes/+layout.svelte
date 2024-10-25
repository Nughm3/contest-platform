<script lang="ts">
	import '../app.css';
	import type { LayoutData } from './$types';
	import { page } from '$app/stores';

	export let data: LayoutData;

	const redirect = $page.url.pathname + $page.url.search;
</script>

<main class="mx-auto my-8 w-full max-w-5xl">
	<header>
		<nav class="mb-4 flex justify-between">
			<ul>
				<a href="/">Contest Platform</a>
			</ul>

			<ul>
				{#if data.user}
					<form method="POST" action="/logout"><button>Log out</button></form>
				{:else}
					<a href="/signup?redirect={redirect}">Sign up</a>
					<a href="/login?redirect={redirect}">Log in</a>
				{/if}
			</ul>
		</nav>
	</header>

	<slot />
</main>
