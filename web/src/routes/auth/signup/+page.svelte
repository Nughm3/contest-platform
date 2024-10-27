<script lang="ts">
	import { enhance } from '$app/forms';
	import { page } from '$app/stores';

	const redirectURL = $page.url.searchParams.get('redirect') ?? '/';
	const redirectParam =
		redirectURL === '/' || redirectURL.slice(0, 5) === '/auth' ? '' : `?redirect=${redirectURL}`;
</script>

<h1>Sign up</h1>

<form method="POST" use:enhance>
	<input type="hidden" name="redirect" value={redirectURL} />

	<label for="username">Username</label>
	<input type="text" id="username" name="username" />

	<label for="password">Password</label>
	<input type="password" id="password" name="password" />

	<p>Already registered? <a href="/auth/login{redirectParam}">Log in!</a></p>

	<button>Sign up</button>
</form>
