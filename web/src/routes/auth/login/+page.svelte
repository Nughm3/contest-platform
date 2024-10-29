<script lang="ts">
	import { enhance } from '$app/forms';
	import { page } from '$app/stores';
	import { Input, Label, Helper, Heading } from 'flowbite-svelte';
	import CircleX from 'lucide-svelte/icons/circle-x';
	import type { ActionData } from './$types';

	interface Props {
		form: ActionData;
	}

	let { form }: Props = $props();

	const redirectURL = $page.url.searchParams.get('redirect') ?? '/';
	const redirectParam =
		redirectURL === '/' || redirectURL.slice(0, 5) === '/auth' ? '' : `?redirect=${redirectURL}`;

	let username = $state('');
	let password = $state('');
	let disabled = $derived(username === '' || password === '');
</script>

<Heading tag="h2" class="mb-6">Log in</Heading>

<form method="POST" use:enhance>
	<input type="hidden" name="redirect" value={redirectURL} />

	<div class="mb-6">
		<Label for="username" class="mb-2">Username</Label>
		<Input name="username" id="username" bind:value={username} />
	</div>

	<div class="mb-6">
		<Label for="password" class="mb-2">Password</Label>
		<Input type="password" name="password" id="password" bind:value={password} />
	</div>

	<Helper class="mb-6">
		No account?
		<a href="/auth/signup{redirectParam}" class="font-medium text-primary-600 hover:underline">
			Sign up!
		</a>
	</Helper>

	{#if form?.error}
		<Helper color="red" class="text-md mb-6 flex items-center space-x-1">
			<CircleX size="20" class="inline-block" />
			<span>{form?.error}</span>
		</Helper>
	{/if}

	<Input type="submit" {disabled} value="Log in" />
</form>
