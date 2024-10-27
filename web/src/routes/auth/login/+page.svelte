<script lang="ts">
	import { enhance } from '$app/forms';
	import { page } from '$app/stores';

	const redirectURL = $page.url.searchParams.get('redirect') ?? '/';
	const redirectParam =
		redirectURL === '/' || redirectURL.slice(0, 5) === '/auth' ? '' : `?redirect=${redirectURL}`;
</script>

<h1>Log in</h1>

<form method="post" use:enhance>
	<input type="hidden" name="redirect" value={redirectURL} />

	<label for="username">Username</label>
	<input name="username" id="username" />

	<label for="password">Password</label>
	<input type="password" name="password" id="password" />

	<p>No account? <a href="/auth/signup{redirectParam}">Sign up!</a></p>

	<button>Continue</button>
</form>
