<script lang="ts">
	const scale = [
		{
			token: 'text-display',
			size: '32px',
			lh: '40px',
			weight: 'font-extrabold (800)',
			use: 'Hero numbers, splash screens',
			class: 'text-display font-extrabold'
		},
		{
			token: 'text-h1',
			size: '26px',
			lh: '32px',
			weight: 'font-extrabold (800)',
			use: 'Page titles',
			class: 'text-h1 font-extrabold'
		},
		{
			token: 'text-h2',
			size: '20px',
			lh: '28px',
			weight: 'font-bold (700)',
			use: 'Section headings',
			class: 'text-h2 font-bold'
		},
		{
			token: 'text-h3',
			size: '16px',
			lh: '24px',
			weight: 'font-semibold (600)',
			use: 'Card titles, sub-headings',
			class: 'text-h3 font-semibold'
		},
		{
			token: 'text-body',
			size: '14px',
			lh: '20px',
			weight: 'font-normal (400)',
			use: 'Main body text',
			class: 'text-body'
		},
		{
			token: 'text-small',
			size: '12px',
			lh: '16px',
			weight: 'font-semibold (600)',
			use: 'Metadata, secondary',
			class: 'text-small font-semibold'
		},
		{
			token: 'text-caption',
			size: '11px',
			lh: '16px',
			weight: 'font-semibold (600)',
			use: 'Labels, captions',
			class: 'text-caption font-semibold'
		},
		{
			token: 'text-micro',
			size: '9px',
			lh: '12px',
			weight: 'font-bold (700)',
			use: 'Badge text, uppercase labels',
			class: 'text-micro font-bold uppercase tracking-wide'
		}
	];

	const lineHeights = [
		{ name: '--lh-tight', value: '1.2', use: 'Headings' },
		{ name: '--lh-normal', value: '1.5', use: 'Body text (default)' },
		{ name: '--lh-relaxed', value: '1.7', use: 'Long prose' }
	];

	const weights = [
		{ name: '--fw-regular', value: '400', tw: 'font-normal', use: 'Body text' },
		{ name: '--fw-semi', value: '600', tw: 'font-semibold', use: 'Labels, small text' },
		{ name: '--fw-bold', value: '700', tw: 'font-bold', use: 'Headings' },
		{ name: '--fw-extra', value: '800', tw: 'font-extrabold', use: 'Display, h1' }
	];
</script>

<h1 class="mb-2 text-h1 font-extrabold">Typography Scale</h1>
<p class="mb-8 text-body text-muted-foreground">
	8-step scale based on Major Third (1.25) ratio. Nunito typeface throughout.
	Use ONLY these tokens — avoid raw text-xs/text-sm/text-base.
</p>

<!-- Type Scale -->
<section class="mb-10">
	<h2 class="mb-4 text-h2 font-bold text-muted-foreground">Type Scale</h2>
	<div class="space-y-6">
		{#each scale as s (s.token)}
			<div class="flex flex-col gap-1 border-b border-border/30 pb-4">
				<div class="flex items-baseline gap-3">
					<code class="rounded bg-muted px-1.5 py-0.5 text-caption font-semibold text-api"
						>{s.token}</code
					>
					<span class="text-small text-muted-foreground">{s.size} / {s.lh}</span>
					<span class="text-small text-muted-foreground">{s.weight}</span>
				</div>
				<p class={s.class}>
					Gotong royong untuk masa depan yang lebih baik.
				</p>
				<p class="text-caption text-kayu">{s.use}</p>
			</div>
		{/each}
	</div>
</section>

<!-- Line Heights -->
<section class="mb-10">
	<h2 class="mb-4 text-h2 font-bold text-muted-foreground">Line Heights</h2>
	<div class="grid gap-4 sm:grid-cols-3">
		{#each lineHeights as lh (lh.name)}
			<div class="rounded-lg border border-border/40 bg-card p-4">
				<code class="text-caption font-semibold text-api">{lh.name}</code>
				<p class="mt-2 text-body" style="line-height: {lh.value}">
					Kerjasama membangun kampung bersama tetangga. Saling membantu dan berbagi.
				</p>
				<p class="mt-1 text-micro text-muted-foreground uppercase">{lh.value} — {lh.use}</p>
			</div>
		{/each}
	</div>
</section>

<!-- Font Weights -->
<section class="mb-10">
	<h2 class="mb-4 text-h2 font-bold text-muted-foreground">Font Weights</h2>
	<div class="space-y-3">
		{#each weights as w (w.name)}
			<div class="flex items-baseline gap-4">
				<code class="w-28 text-caption text-muted-foreground">{w.tw}</code>
				<span class="text-h3" style="font-weight: {w.value}">Gotong Royong ({w.value})</span>
				<span class="text-small text-kayu">{w.use}</span>
			</div>
		{/each}
	</div>
</section>
