<script lang="ts">
	import { enhance } from '$app/forms';
</script>

<h1>Register</h1>

<form method="POST" use:enhance>
	<label for="username">Username</label>
	<input name="username" id="username" /><br />
	<label for="password">Password</label>
	<input type="password" name="password" id="password" /><br />
	<button>Register</button>
	<a href="/login">Log in</a>
</form>
