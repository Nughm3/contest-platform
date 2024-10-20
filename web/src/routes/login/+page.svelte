<script lang="ts">
	import { enhance } from '$app/forms';
	import { page } from '$app/stores';

	const redirect = $page.url.searchParams.get('redirect') ?? '/';
</script>

<h1>Log in</h1>

<form method="post" use:enhance>
	<input type="hidden" name="redirect" value={redirect} />

	<label for="username">Username</label>
	<input name="username" id="username" />

	<label for="password">Password</label>
	<input type="password" name="password" id="password" />

	<p>No account? <a href="/signup?redirect={redirect}">Sign up!</a></p>

	<button>Continue</button>
</form>
