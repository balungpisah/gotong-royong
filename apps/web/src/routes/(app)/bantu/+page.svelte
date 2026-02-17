<script lang="ts">
	import { PlaceholderPanel } from '$lib/components';
	import { m } from '$lib/paraglide/messages';
</script>

<PlaceholderPanel
	badge={m.page_placeholder_badge()}
	title={m.page_bantu_title()}
	description={m.page_bantu_description()}
	actions={[
		{ href: '/terlibat', label: m.shell_nav_terlibat(), variant: 'outline' },
		{ href: '/notifikasi', label: m.shell_nav_notifikasi() }
	]}
/>
