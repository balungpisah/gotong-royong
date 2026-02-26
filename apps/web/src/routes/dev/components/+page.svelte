<script lang="ts">
	const foundations = [
		{
			name: 'Tokens',
			href: '/dev/components/tokens',
			desc: 'Color palettes, spacing grid, radius, shadows — the atomic layer'
		},
		{
			name: 'Typography',
			href: '/dev/components/typography',
			desc: '8-step type scale (display → micro), weights, line heights'
		}
	];

	const families = [
		{
			name: 'Button',
			href: '/dev/components/button',
			desc: '14 variants × 7 sizes (base + track + vault/siaga + pill)'
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
		{
			name: 'Select',
			href: '/dev/components/select',
			desc: 'Native select wrapper matching Input styling'
		},
		{
			name: 'FormField',
			href: '/dev/components/form-field',
			desc: 'Compound: Label + Input/Textarea/Select + Hint/Error'
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
		{
			name: 'RadioGroup',
			href: '/dev/components/radio-group',
			desc: 'Radio buttons — vertical/horizontal, disabled, with descriptions'
		},
		{
			name: 'Checkbox',
			href: '/dev/components/checkbox',
			desc: 'Checkbox — checked, indeterminate, disabled states'
		},
		{ name: 'Tooltip', href: '/dev/components/tooltip', desc: 'Tooltip with DNA styling' },
		{
			name: 'Separator',
			href: '/dev/components/separator',
			desc: 'Horizontal and vertical dividers'
		},
		{
			name: 'Card',
			href: '/dev/components/card',
			desc: '5 variants × 3 paddings + interactive states'
		},
		{ name: 'Dialog', href: '/dev/components/dialog', desc: 'Modal dialog with overlay' },
		{
			name: 'Feed Cards',
			href: '/dev/components/feed-cards',
			desc: 'Registry-driven witness/system matrix from JSON scenarios'
		}
	];
</script>

<h1 class="mb-2 text-h1 font-extrabold text-foreground">Design System — Component Gallery</h1>
<p class="mb-8 text-body text-muted-foreground">
	Design tokens, typography, and {families.length} component families. Dev-only route.
</p>

<h2 class="mb-3 text-h3 font-semibold text-muted-foreground">Foundations</h2>
<div class="mb-8 grid gap-4 sm:grid-cols-2">
	{#each foundations as f (f.href)}
		<a
			href={f.href}
			class="group rounded-lg border-2 border-dashed border-api/30 bg-bara/30 p-4 transition-colors hover:border-api/60 hover:bg-bara/50"
		>
			<h2 class="mb-1 text-h3 font-semibold text-foreground group-hover:text-api">
				{f.name}
			</h2>
			<p class="text-small text-muted-foreground">{f.desc}</p>
		</a>
	{/each}
</div>

<h2 class="mb-3 text-h3 font-semibold text-muted-foreground">Components</h2>
<div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
	{#each families as family (family.href)}
		<a
			href={family.href}
			class="group rounded-lg border border-border bg-card p-4 transition-colors hover:border-primary/40 hover:bg-accent/30"
		>
			<h2 class="mb-1 text-h3 font-semibold text-foreground group-hover:text-primary">
				{family.name}
			</h2>
			<p class="text-small text-muted-foreground">{family.desc}</p>
		</a>
	{/each}
</div>
