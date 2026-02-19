<script lang="ts">
	import Shield from '@lucide/svelte/icons/shield';
	import Star from '@lucide/svelte/icons/star';
	import Award from '@lucide/svelte/icons/award';
	import Eye from '@lucide/svelte/icons/eye';
	import HandHeart from '@lucide/svelte/icons/hand-heart';
	import { motion } from '@humanspeak/svelte-motion';

	interface Props {
		userId?: string | null;
	}

	let { userId = null }: Props = $props();
</script>

<!--
	SelfProfile — person profile panel for the context box.
	Shows user identity, contribution stats, tandang reputation,
	and Octalysis engagement metrics.
	Real data will be wired later — this is the layout skeleton.
-->

<div class="flex h-full flex-col">
	<!-- Profile header -->
	<div class="border-b border-border/20 px-5 py-5">
		<motion.div
			class="flex items-center gap-4"
			initial={{ opacity: 0, x: -8 }}
			animate={{ opacity: 1, x: 0 }}
			transition={{ duration: 0.3 }}
		>
			<!-- Avatar -->
			<div class="relative">
				<div class="size-14 rounded-full bg-gradient-to-br from-primary/30 to-primary/10 p-0.5">
					<div class="flex size-full items-center justify-center rounded-full bg-card text-[var(--fs-h2)] font-bold text-primary">
						AH
					</div>
				</div>
				<!-- Online indicator -->
				<div class="absolute bottom-0 right-0 size-3.5 rounded-full border-2 border-card bg-green-500"></div>
			</div>
			<div class="min-w-0 flex-1">
				<h2 class="truncate text-[var(--fs-body)] font-bold text-foreground">Ahmad Hidayat</h2>
				<p class="text-[var(--fs-caption)] text-muted-foreground">RT 05 · Warga aktif sejak 2024</p>
			</div>
		</motion.div>

		<!-- Role badges -->
		<motion.div
			class="mt-3 flex flex-wrap gap-1.5"
			initial={{ opacity: 0 }}
			animate={{ opacity: 1 }}
			transition={{ duration: 0.3, delay: 0.1 }}
		>
			<span class="inline-flex items-center gap-1 rounded-full bg-primary/10 px-2.5 py-0.5 text-[var(--fs-caption)] font-medium text-primary">
				<Shield class="size-3" />
				Saksi Terpercaya
			</span>
			<span class="inline-flex items-center gap-1 rounded-full bg-amber-100 px-2.5 py-0.5 text-[var(--fs-caption)] font-medium text-amber-700">
				<Star class="size-3" />
				Level 3
			</span>
		</motion.div>
	</div>

	<!-- Profile content -->
	<div class="flex-1 overflow-y-auto px-5 py-4">
		<!-- Contribution stats -->
		<motion.div
			initial={{ opacity: 0, y: 8 }}
			animate={{ opacity: 1, y: 0 }}
			transition={{ duration: 0.3, delay: 0.15 }}
		>
			<h3 class="text-[var(--fs-small)] font-semibold text-foreground">Kontribusi</h3>
			<div class="mt-2 grid grid-cols-3 gap-2">
				<div class="rounded-lg bg-muted/20 p-2.5 text-center">
					<p class="text-[var(--fs-h2)] font-bold text-foreground">12</p>
					<p class="text-[10px] text-muted-foreground">Laporan</p>
				</div>
				<div class="rounded-lg bg-muted/20 p-2.5 text-center">
					<p class="text-[var(--fs-h2)] font-bold text-foreground">8</p>
					<p class="text-[10px] text-muted-foreground">Saksi</p>
				</div>
				<div class="rounded-lg bg-muted/20 p-2.5 text-center">
					<p class="text-[var(--fs-h2)] font-bold text-foreground">5</p>
					<p class="text-[10px] text-muted-foreground">Resolusi</p>
				</div>
			</div>
		</motion.div>

		<!-- Tandang reputation -->
		<motion.div
			class="mt-4"
			initial={{ opacity: 0, y: 8 }}
			animate={{ opacity: 1, y: 0 }}
			transition={{ duration: 0.3, delay: 0.2 }}
		>
			<h3 class="text-[var(--fs-small)] font-semibold text-foreground">Reputasi Tandang</h3>
			<p class="mt-0.5 text-[var(--fs-caption)] text-muted-foreground">Sinyal yang diterima dari komunitas</p>
			<div class="mt-3 space-y-2.5">
				{#each [
					{ label: 'Vouch', received: 18, icon: HandHeart, color: 'text-green-600 bg-green-50' },
					{ label: 'Bagus!', received: 14, icon: Star, color: 'text-purple-600 bg-purple-50' },
					{ label: 'Proof of Resolve', received: 7, icon: Award, color: 'text-blue-600 bg-blue-50' },
					{ label: 'Skeptis', received: 2, icon: Eye, color: 'text-amber-600 bg-amber-50' },
				] as signal}
					<div class="flex items-center gap-3">
						<div class="flex size-7 items-center justify-center rounded-md {signal.color}">
							<signal.icon class="size-3.5" />
						</div>
						<div class="min-w-0 flex-1">
							<div class="flex items-center justify-between">
								<span class="text-[var(--fs-caption)] font-medium text-foreground">{signal.label}</span>
								<span class="text-[var(--fs-caption)] font-bold text-foreground">{signal.received}</span>
							</div>
							<div class="mt-1 h-1.5 w-full rounded-full bg-muted/30">
								<div
									class="h-full rounded-full bg-primary/50 transition-all duration-500"
									style="width: {Math.min((signal.received / 20) * 100, 100)}%"
								></div>
							</div>
						</div>
					</div>
				{/each}
			</div>
		</motion.div>

		<!-- Octalysis engagement (placeholder) -->
		<motion.div
			class="mt-4 rounded-xl border border-border/30 bg-muted/10 p-4"
			initial={{ opacity: 0, y: 12 }}
			animate={{ opacity: 1, y: 0 }}
			transition={{ duration: 0.35, delay: 0.3 }}
		>
			<h3 class="text-[var(--fs-small)] font-semibold text-foreground">Engagement Drivers</h3>
			<p class="mt-0.5 text-[var(--fs-caption)] text-muted-foreground">Octalysis framework</p>
			<div class="mt-3 space-y-2">
				{#each [
					{ core: 'Epic Meaning', score: 85 },
					{ core: 'Accomplishment', score: 72 },
					{ core: 'Empowerment', score: 60 },
					{ core: 'Social Influence', score: 78 },
					{ core: 'Unpredictability', score: 45 },
				] as drive}
					<div class="flex items-center gap-2">
						<span class="w-28 text-[var(--fs-caption)] text-muted-foreground">{drive.core}</span>
						<div class="h-1.5 flex-1 rounded-full bg-muted/30">
							<div
								class="h-full rounded-full bg-primary transition-all duration-500"
								style="width: {drive.score}%; opacity: {0.4 + (drive.score / 100) * 0.6}"
							></div>
						</div>
						<span class="w-6 text-right text-[10px] font-medium text-foreground">{drive.score}</span>
					</div>
				{/each}
			</div>
		</motion.div>

		<!-- Recent activity -->
		<motion.div
			class="mt-4"
			initial={{ opacity: 0 }}
			animate={{ opacity: 1 }}
			transition={{ duration: 0.3, delay: 0.4 }}
		>
			<h3 class="text-[var(--fs-small)] font-semibold text-foreground">Aktivitas Terbaru</h3>
			<div class="mt-2 space-y-1.5">
				{#each [
					{ text: 'Memberi Vouch pada laporan banjir', time: '2 jam lalu' },
					{ text: 'Menjadi saksi untuk jalan rusak RT 05', time: '1 hari lalu' },
					{ text: 'Menyelesaikan penggalangan dana', time: '3 hari lalu' },
				] as activity}
					<div class="flex items-start gap-2 rounded-lg px-2 py-1.5">
						<div class="mt-1.5 size-1.5 shrink-0 rounded-full bg-primary/40"></div>
						<div>
							<p class="text-[var(--fs-caption)] leading-relaxed text-foreground/80">{activity.text}</p>
							<p class="text-[10px] text-muted-foreground">{activity.time}</p>
						</div>
					</div>
				{/each}
			</div>
		</motion.div>
	</div>
</div>
