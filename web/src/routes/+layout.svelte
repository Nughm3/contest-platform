<script lang="ts">
	import '../app.css';
	import { page } from '$app/stores';
	import { enhance } from '$app/forms';
	import type { LayoutData } from './$types';
	import type { Snippet } from 'svelte';
	import { Trophy, ChevronDown } from 'lucide-svelte';

	interface Props {
		data: LayoutData;
		children: Snippet;
	}

	let { data, children }: Props = $props();

	const redirectURL = $page.url.pathname + $page.url.search;
	const redirectParam =
		redirectURL === '/' || redirectURL.slice(0, 5) === '/auth' ? '' : `?redirect=${redirectURL}`;
</script>

<svelte:head>
	<title>Contest Platform</title>
</svelte:head>

<div class="min-w-screen min-h-screen p-4">
	<main class="mx-auto max-w-5xl">
		<header>
			<nav class="mb-8 mt-4 flex items-center justify-between">
				<div class="flex items-center space-x-2">
					<a href="/" class="text-2xl font-bold hover:underline">Contest Platform</a>
					{#if data.contest}
						<span class="text-slate-600">|</span>
						<a href={`/contest/${data.contest.slug}`} class="text-slate-800 hover:underline"
							>{data.contest.name}</a
						>
					{/if}
				</div>

				<div class="flex items-center space-x-2">
					{#if data.username}
						{#if data.contest}
							<a
								href={'/contest/' + data.contest.slug + '/leaderboard'}
								class="flex items-center space-x-1 text-slate-600"
							>
								<Trophy size="20" class="inline-block" />
								<span class="hover:underline">Leaderboard</span>
							</a>
							<span class="text-slate-600">|</span>
						{/if}
						{#if data.isAdmin}
							<a
								href="/admin"
								class="rounded-md bg-red-500 p-1 text-xs font-medium text-white hover:bg-red-600"
								>ADMIN</a
							>
						{/if}
						<form method="POST" action="/auth/logout" class="inline-block" use:enhance>
							<button class="flex items-center space-x-1">
								<span class="hover:underline">Log out</span>
								<ChevronDown size="20" class="inline-block" />
							</button>
						</form>
					{:else}
						<a href="/auth/signup{redirectParam}" class="hover:underline">Sign up</a>
						<a href="/auth/login{redirectParam}" class="hover:underline">Log in</a>
					{/if}
				</div>
			</nav>
		</header>

		{@render children()}
	</main>
</div>
