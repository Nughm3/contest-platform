<script lang="ts">
	import { enhance } from '$app/forms';
	import { page } from '$app/stores';
	import CircleX from 'lucide-svelte/icons/circle-x';
	import { Input, Label, Helper, Heading } from 'flowbite-svelte';
	import type { ActionData } from './$types';

	interface Props {
		form: ActionData;
	}

	let { form }: Props = $props();

	interface ValidationColor {
		color: 'red' | 'green';
	}

	const redirectURL = $page.url.searchParams.get('redirect') ?? '/';
	const redirectParam =
		redirectURL === '/' || redirectURL.slice(0, 5) === '/auth' ? '' : `?redirect=${redirectURL}`;

	let username = $state(form?.username?.toString() ?? '');
	let usernameError = $derived(
		username.length < 3 || username.length > 31 || !/^[a-zA-Z0-9_-]+$/.test(username)
	);
	let usernameColor: ValidationColor | null = $derived(
		username !== '' ? { color: usernameError ? 'red' : 'green' } : null
	);

	let password = $state('');
	let passwordError = $derived(password.length < 6 || password.length > 255);
	let passwordColor: ValidationColor | null = $derived(
		password !== '' ? { color: passwordError ? 'red' : 'green' } : null
	);

	let confirm = $state('');
	let confirmError = $derived(password !== '' && confirm !== password);
	let confirmColor: ValidationColor | null = $derived(
		password !== '' ? { color: confirmError ? 'red' : 'green' } : null
	);

	let disabled = $derived(usernameError || passwordError || confirmError);
</script>

<Heading tag="h2" class="mb-6">Sign up</Heading>

<form method="POST" use:enhance>
	<input type="hidden" name="redirect" value={redirectURL} />

	<div class="mb-6">
		<Label for="username" class="mb-2" {...usernameColor}>Username</Label>
		<Input
			type="text"
			id="username"
			name="username"
			bind:value={username}
			required
			{...usernameColor}
			class="mb-2"
		/>
		{#if usernameError && username !== ''}
			<Helper color="red" class="flex items-center space-x-1">
				<CircleX size="16" class="inline-block" />
				<span>
					Must be 3-31 characters long, containing alphanumeric characters, hyphens and underscores
				</span>
			</Helper>
		{/if}
	</div>

	<div class="mb-6">
		<Label for="password" {...passwordColor} class="mb-2">Password</Label>
		<Input
			type="password"
			id="password"
			name="password"
			bind:value={password}
			required
			{...passwordColor}
			class="mb-2"
		/>
		{#if passwordError && password !== ''}
			<Helper color="red" class="flex items-center space-x-1">
				<CircleX size="16" class="inline-block" />
				<span>Must be 6-255 characters long</span>
			</Helper>
		{/if}
	</div>

	<div class="mb-6">
		<Label for="password" {...confirmColor} class="mb-2">Confirm password</Label>
		<Input type="password" bind:value={confirm} required {...confirmColor} class="mb-2" />
		{#if confirmError}
			<Helper color="red" class="flex items-center space-x-1">
				<CircleX size="16" class="inline-block" />
				<span>Passwords do not match</span>
			</Helper>
		{/if}
	</div>

	<Helper class="mb-6">
		Already registered?
		<a href="/auth/login{redirectParam}" class="font-medium text-primary-600 hover:underline">
			Log in!
		</a>
	</Helper>

	{#if form?.error}
		<Helper color="red" class="text-md mb-6 flex items-center space-x-1">
			<CircleX size="20" class="inline-block" />
			<span>{form?.error}</span>
		</Helper>
	{/if}

	<Input type="submit" {disabled} value="Sign up" />
</form>
