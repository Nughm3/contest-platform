<script lang="ts">
	import { enhance } from '$app/forms';
</script>

<form method="post" enctype="multipart/form-data" use:enhance>
	<label for="language"></label>
	<select name="language" id="language">
		<option>C 99</option>
		<option>C++ 17</option>
		<option>Python 3</option>
	</select>

	<label for="code">Upload code</label>
	<input type="file" name="code" id="code" />

	<input type="submit" value="Submit" />
</form>
