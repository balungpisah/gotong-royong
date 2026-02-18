<script lang="ts">
	const families = [
		{
			name: 'Button',
			href: '/dev/components/button',
			desc: '13 variants × 3 sizes (base + track + vault/siaga)'
		},
		{
			name: 'Badge',
			href: '/dev/components/badge',
			desc: '16 variants (base + track + semantic + stepper + special)'
		},
		{
			name: 'Input',
			href: '/dev/components/input',
			desc: '4 states + label/hint/error sub-components + textarea'
		},
		{ name: 'Avatar', href: '/dev/components/avatar', desc: '5 sizes + group + tier overlay' },
		{
			name: 'Pill',
			href: '/dev/components/pill',
			desc: '10 variants (active, default, success, removable, 5 tracks)'
		},
		{
			name: 'Status Indicator',
			href: '/dev/components/status-indicator',
			desc: '6 states with dot + label'
		},
		{
			name: 'Progress',
			href: '/dev/components/progress',
			desc: 'Progress bar + confidence bar + track variants'
		},
		{ name: 'Switch', href: '/dev/components/switch', desc: 'Toggle with DNA styling' },
		{ name: 'Tooltip', href: '/dev/components/tooltip', desc: 'Tooltip with DNA styling' },
		{
			name: 'Separator',
			href: '/dev/components/separator',
			desc: 'Horizontal and vertical dividers'
		},
		{ name: 'Card', href: '/dev/components/card', desc: '7-part card system' },
		{ name: 'Dialog', href: '/dev/components/dialog', desc: 'Modal dialog with overlay' }
	];
</script>

<h1 class="mb-2 text-2xl font-bold text-foreground">Design DNA — Component Gallery</h1>
<p class="mb-8 text-sm text-muted-foreground">
	12 atom families from Design DNA §4. Dev-only — this route is gated behind the dev environment.
</p>

<div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
	{#each families as family (family.href)}
		<a
			href={family.href}
			class="group rounded-lg border border-border bg-card p-4 transition-colors hover:border-primary/40 hover:bg-accent/30"
		>
			<h2 class="mb-1 text-base font-semibold text-foreground group-hover:text-primary">
				{family.name}
			</h2>
			<p class="text-xs text-muted-foreground">{family.desc}</p>
		</a>
	{/each}
</div>
