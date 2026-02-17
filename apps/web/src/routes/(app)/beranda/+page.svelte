<script lang="ts">
	import { PlaceholderPanel } from '$lib/components';
	import { m } from '$lib/paraglide/messages';
</script>

<PlaceholderPanel
	badge={m.page_placeholder_badge()}
	title={m.page_beranda_title()}
	description={m.page_beranda_description()}
	actions={[
		{ href: '/terlibat', label: m.shell_nav_terlibat() },
		{ href: '/profil', label: m.shell_nav_profil(), variant: 'outline' }
	]}
/>
