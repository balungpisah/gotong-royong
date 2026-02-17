<script lang="ts">
	import { PlaceholderPanel } from '$lib/components';
	import { m } from '$lib/paraglide/messages';
</script>

<PlaceholderPanel
	badge={m.page_placeholder_badge()}
	title={m.page_profil_title()}
	description={m.page_profil_description()}
	actions={[
		{ href: '/beranda', label: m.shell_nav_beranda(), variant: 'outline' },
		{ href: '/bantu', label: m.shell_nav_bantu() }
	]}
/>
