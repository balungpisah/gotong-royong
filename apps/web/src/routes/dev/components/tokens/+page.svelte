<script lang="ts">
	const tanah = [
		{ name: '--c-tanah-gelap', tw: 'bg-tanah-gelap', hex: '#3E2723', label: 'Tanah Gelap' },
		{ name: '--c-tanah', tw: 'bg-tanah', hex: '#5D4037', label: 'Tanah' },
		{ name: '--c-kayu', tw: 'bg-kayu', hex: '#8D6E63', label: 'Kayu' },
		{ name: '--c-pasir', tw: 'bg-pasir', hex: '#A1887F', label: 'Pasir' },
		{ name: '--c-batu', tw: 'bg-batu', hex: '#D7CCC8', label: 'Batu' },
		{ name: '--c-kapas', tw: 'bg-kapas', hex: '#F5EDE3', label: 'Kapas' },
		{ name: '--c-susu', tw: 'bg-susu', hex: '#FFFBF5', label: 'Susu' }
	];

	const api = [
		{ name: '--c-api', tw: 'bg-api', hex: '#C05621', label: 'Api' },
		{ name: '--c-api-terang', tw: 'bg-api-terang', hex: '#D2691E', label: 'Api Terang' },
		{ name: '--c-api-dalam', tw: 'bg-api-dalam', hex: '#A0461A', label: 'Api Dalam' },
		{ name: '--c-bara', tw: 'bg-bara', hex: '#FFF3E0', label: 'Bara' }
	];

	const semantik = [
		{ name: '--c-berhasil', tw: 'bg-berhasil', hex: '#2E7D32', label: 'Berhasil' },
		{ name: '--c-berhasil-lembut', tw: 'bg-berhasil-lembut', hex: '#E8F5E9', label: 'Berhasil Lembut' },
		{ name: '--c-peringatan', tw: 'bg-peringatan', hex: '#E65100', label: 'Peringatan' },
		{ name: '--c-peringatan-lembut', tw: 'bg-peringatan-lembut', hex: '#FFF3E0', label: 'Peringatan Lembut' },
		{ name: '--c-bahaya', tw: 'bg-bahaya', hex: '#C62828', label: 'Bahaya' },
		{ name: '--c-bahaya-lembut', tw: 'bg-bahaya-lembut', hex: '#FFEBEE', label: 'Bahaya Lembut' },
		{ name: '--c-keterangan', tw: 'bg-keterangan', hex: '#4E342E', label: 'Keterangan' },
		{ name: '--c-keterangan-lembut', tw: 'bg-keterangan-lembut', hex: '#EFEBE9', label: 'Keterangan Lembut' }
	];

	const tracks = [
		{ name: '--t-tuntaskan', tw: 'bg-tuntaskan', hex: '#C05621', label: 'Tuntaskan' },
		{ name: '--t-tuntaskan-soft', tw: 'bg-tuntaskan-soft', hex: '#FFF3E0', label: 'Soft' },
		{ name: '--t-wujudkan', tw: 'bg-wujudkan', hex: '#2E7D32', label: 'Wujudkan' },
		{ name: '--t-wujudkan-soft', tw: 'bg-wujudkan-soft', hex: '#E8F5E9', label: 'Soft' },
		{ name: '--t-telusuri', tw: 'bg-telusuri', hex: '#6A1B9A', label: 'Telusuri' },
		{ name: '--t-telusuri-soft', tw: 'bg-telusuri-soft', hex: '#F3E5F5', label: 'Soft' },
		{ name: '--t-rayakan', tw: 'bg-rayakan', hex: '#F57F17', label: 'Rayakan' },
		{ name: '--t-rayakan-soft', tw: 'bg-rayakan-soft', hex: '#FFF8E1', label: 'Soft' },
		{ name: '--t-musyawarah', tw: 'bg-musyawarah', hex: '#4E342E', label: 'Musyawarah' },
		{ name: '--t-musyawarah-soft', tw: 'bg-musyawarah-soft', hex: '#EFEBE9', label: 'Soft' }
	];

	const spacing = [
		{ name: '--sp-1', value: '4px', tw: 'w-1' },
		{ name: '--sp-2', value: '8px', tw: 'w-2' },
		{ name: '--sp-3', value: '12px', tw: 'w-3' },
		{ name: '--sp-4', value: '16px', tw: 'w-4' },
		{ name: '--sp-5', value: '20px', tw: 'w-5' },
		{ name: '--sp-6', value: '24px', tw: 'w-6' },
		{ name: '--sp-8', value: '32px', tw: 'w-8' },
		{ name: '--sp-10', value: '40px', tw: 'w-10' },
		{ name: '--sp-12', value: '48px', tw: 'w-12' },
		{ name: '--sp-16', value: '64px', tw: 'w-16' }
	];

	const radii = [
		{ name: '--r-sm', value: '6px', tw: 'rounded-sm' },
		{ name: '--r-md', value: '10px', tw: 'rounded-md' },
		{ name: '--r-lg', value: '14px', tw: 'rounded-lg' },
		{ name: '--r-xl', value: '20px', tw: 'rounded-xl' },
		{ name: '--r-full', value: '9999px', tw: 'rounded-full' }
	];

	const shadows = [
		{ name: '--shadow-sm', tw: 'shadow-sm', label: 'Small' },
		{ name: '--shadow-md', tw: 'shadow-md', label: 'Medium' },
		{ name: '--shadow-lg', tw: 'shadow-lg', label: 'Large' },
		{ name: '--shadow-xl', tw: 'shadow-xl', label: 'Extra Large' }
	];
</script>

<h1 class="mb-2 text-h1 font-extrabold">Design Tokens</h1>
<p class="mb-8 text-body text-muted-foreground">
	The foundational values that drive every component. Change a token, change the entire platform.
</p>

<!-- Colors: Tanah -->
<section class="mb-10">
	<h2 class="mb-4 text-h2 font-bold text-muted-foreground">Tanah (Earth Foundation)</h2>
	<div class="flex flex-wrap gap-3">
		{#each tanah as c (c.name)}
			<div class="flex flex-col items-center gap-1.5">
				<div class="size-14 rounded-lg border border-border/40 {c.tw}"></div>
				<span class="text-caption font-semibold">{c.label}</span>
				<code class="text-micro text-muted-foreground">{c.hex}</code>
			</div>
		{/each}
	</div>
</section>

<!-- Colors: Api -->
<section class="mb-10">
	<h2 class="mb-4 text-h2 font-bold text-muted-foreground">Api (Action Fire)</h2>
	<div class="flex flex-wrap gap-3">
		{#each api as c (c.name)}
			<div class="flex flex-col items-center gap-1.5">
				<div class="size-14 rounded-lg border border-border/40 {c.tw}"></div>
				<span class="text-caption font-semibold">{c.label}</span>
				<code class="text-micro text-muted-foreground">{c.hex}</code>
			</div>
		{/each}
	</div>
</section>

<!-- Colors: Semantik -->
<section class="mb-10">
	<h2 class="mb-4 text-h2 font-bold text-muted-foreground">Semantik (System Feedback)</h2>
	<div class="flex flex-wrap gap-3">
		{#each semantik as c (c.name)}
			<div class="flex flex-col items-center gap-1.5">
				<div class="size-14 rounded-lg border border-border/40 {c.tw}"></div>
				<span class="text-caption font-semibold">{c.label}</span>
				<code class="text-micro text-muted-foreground">{c.hex}</code>
			</div>
		{/each}
	</div>
</section>

<!-- Colors: Tracks -->
<section class="mb-10">
	<h2 class="mb-4 text-h2 font-bold text-muted-foreground">Track Accents</h2>
	<div class="flex flex-wrap gap-3">
		{#each tracks as c (c.name)}
			<div class="flex flex-col items-center gap-1.5">
				<div class="size-14 rounded-lg border border-border/40 {c.tw}"></div>
				<span class="text-caption font-semibold">{c.label}</span>
				<code class="text-micro text-muted-foreground">{c.hex}</code>
			</div>
		{/each}
	</div>
</section>

<!-- Spacing -->
<section class="mb-10">
	<h2 class="mb-4 text-h2 font-bold text-muted-foreground">Spacing (4px Grid)</h2>
	<div class="space-y-2">
		{#each spacing as s (s.name)}
			<div class="flex items-center gap-3">
				<code class="w-20 text-caption text-muted-foreground">{s.name}</code>
				<div class="h-4 rounded bg-api/70" style="width: {s.value}"></div>
				<span class="text-small text-muted-foreground">{s.value}</span>
			</div>
		{/each}
	</div>
</section>

<!-- Radius -->
<section class="mb-10">
	<h2 class="mb-4 text-h2 font-bold text-muted-foreground">Border Radius</h2>
	<div class="flex flex-wrap gap-4">
		{#each radii as r (r.name)}
			<div class="flex flex-col items-center gap-1.5">
				<div class="size-16 border-2 border-api bg-api/10" style="border-radius: {r.value}"></div>
				<span class="text-caption font-semibold">{r.tw}</span>
				<code class="text-micro text-muted-foreground">{r.value}</code>
			</div>
		{/each}
	</div>
</section>

<!-- Shadows -->
<section class="mb-10">
	<h2 class="mb-4 text-h2 font-bold text-muted-foreground">Shadows (Brown-Tinted)</h2>
	<div class="flex flex-wrap gap-6">
		{#each shadows as s (s.name)}
			<div class="flex flex-col items-center gap-1.5">
				<div class="size-20 rounded-xl bg-card {s.tw}"></div>
				<span class="text-caption font-semibold">{s.label}</span>
				<code class="text-micro text-muted-foreground">{s.tw}</code>
			</div>
		{/each}
	</div>
</section>
