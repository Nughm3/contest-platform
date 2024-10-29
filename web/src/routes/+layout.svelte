<script lang="ts">
	import '../app.css';
	import { page } from '$app/stores';
	import { enhance } from '$app/forms';
	import ChevronDown from 'lucide-svelte/icons/chevron-down';
	import User from 'lucide-svelte/icons/user';
	import { Button, Dropdown, DropdownItem } from 'flowbite-svelte';
	import type { LayoutData } from './$types';
	import type { Snippet } from 'svelte';

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
						<a href={`/contest/${data.contest.slug}`} class="text-slate-600 hover:underline">
							{data.contest.name}
						</a>
					{/if}
				</div>

				<div class="flex items-center space-x-4">
					{#if data.username}
						{#if data.contest}{/if}

						<div>
							<Button color="alternative" class="flex items-center space-x-1">
								<User size="20" class="inline-block" />
								<strong>{data.username}</strong>
								<ChevronDown size="20" class="inline-block" />
							</Button>
							<Dropdown>
								{#if data.isAdmin}
									<DropdownItem><a href="/admin">Admin</a></DropdownItem>
								{/if}
								<DropdownItem>
									<form method="POST" action="/auth/logout" class="inline-block" use:enhance>
										<input type="submit" value="Log out" />
									</form>
								</DropdownItem>
							</Dropdown>
						</div>
					{:else}
						<a href="/auth/login{redirectParam}"><Button color="alternative">Log in</Button></a>
						<a href="/auth/signup{redirectParam}"><Button>Sign up</Button></a>
					{/if}
				</div>
			</nav>
		</header>

		{@render children()}
	</main>
</div>
