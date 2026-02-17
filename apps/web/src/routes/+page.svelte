<script lang="ts">
	import { m } from '$lib/paraglide/messages';
</script>

<h1>{m.home_title()}</h1>
<p>
	{m.home_docs_prefix()}
	<a href="https://svelte.dev/docs/kit">{m.home_docs_link_label()}</a>
	{m.home_docs_suffix()}
</p>
