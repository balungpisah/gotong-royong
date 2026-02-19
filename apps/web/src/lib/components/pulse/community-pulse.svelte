<script lang="ts">
	import Activity from '@lucide/svelte/icons/activity';
	import Users from '@lucide/svelte/icons/users';
	import TrendingUp from '@lucide/svelte/icons/trending-up';
	import MessageCircle from '@lucide/svelte/icons/message-circle';
	import Heart from '@lucide/svelte/icons/heart';
	import { motion } from '@humanspeak/svelte-motion';
</script>

<!--
	CommunityPulse — overview dashboard for the community context box.
	Shows aggregate stats, participation metrics, and health indicators.
	This is the "rest state" of the context box when pinned and nothing selected.
	Charts and real data will be wired in later — this is the layout skeleton.
-->

<div class="flex h-full flex-col">
	<!-- Header -->
	<div class="flex items-center gap-3 border-b border-border/20 px-5 py-4">
		<div class="flex size-9 items-center justify-center rounded-lg bg-primary/10 text-primary">
			<Activity class="size-5" />
		</div>
		<div>
			<h2 class="text-[var(--fs-body)] font-bold text-foreground">Pulse Komunitas</h2>
			<p class="text-[var(--fs-caption)] text-muted-foreground">Ikhtisar kesehatan dan partisipasi</p>
		</div>
	</div>

	<!-- Stats grid -->
	<div class="flex-1 overflow-y-auto px-5 py-4">
		<!-- Quick stats row -->
		<div class="grid grid-cols-2 gap-3">
			<motion.div
				class="stat-card"
				initial={{ opacity: 0, y: 8 }}
				animate={{ opacity: 1, y: 0 }}
				transition={{ duration: 0.3, delay: 0 }}
			>
				<div class="flex items-center gap-2 text-muted-foreground">
					<Users class="size-3.5" />
					<span class="text-[var(--fs-caption)]">Saksi Aktif</span>
				</div>
				<p class="mt-1 text-[var(--fs-h2)] font-bold text-foreground">24</p>
				<p class="text-[var(--fs-caption)] text-green-600">+3 minggu ini</p>
			</motion.div>

			<motion.div
				class="stat-card"
				initial={{ opacity: 0, y: 8 }}
				animate={{ opacity: 1, y: 0 }}
				transition={{ duration: 0.3, delay: 0.05 }}
			>
				<div class="flex items-center gap-2 text-muted-foreground">
					<MessageCircle class="size-3.5" />
					<span class="text-[var(--fs-caption)]">Pesan Hari Ini</span>
				</div>
				<p class="mt-1 text-[var(--fs-h2)] font-bold text-foreground">47</p>
				<p class="text-[var(--fs-caption)] text-muted-foreground">12 percakapan</p>
			</motion.div>

			<motion.div
				class="stat-card"
				initial={{ opacity: 0, y: 8 }}
				animate={{ opacity: 1, y: 0 }}
				transition={{ duration: 0.3, delay: 0.1 }}
			>
				<div class="flex items-center gap-2 text-muted-foreground">
					<TrendingUp class="size-3.5" />
					<span class="text-[var(--fs-caption)]">Tingkat Resolusi</span>
				</div>
				<p class="mt-1 text-[var(--fs-h2)] font-bold text-foreground">73%</p>
				<div class="mt-1.5 h-1.5 w-full rounded-full bg-muted/40">
					<div class="h-full w-[73%] rounded-full bg-green-500 transition-all duration-500"></div>
				</div>
			</motion.div>

			<motion.div
				class="stat-card"
				initial={{ opacity: 0, y: 8 }}
				animate={{ opacity: 1, y: 0 }}
				transition={{ duration: 0.3, delay: 0.15 }}
			>
				<div class="flex items-center gap-2 text-muted-foreground">
					<Heart class="size-3.5" />
					<span class="text-[var(--fs-caption)]">Sinyal Tandang</span>
				</div>
				<p class="mt-1 text-[var(--fs-h2)] font-bold text-foreground">156</p>
				<p class="text-[var(--fs-caption)] text-muted-foreground">minggu ini</p>
			</motion.div>
		</div>

		<!-- Participation chart placeholder -->
		<motion.div
			class="mt-4 rounded-xl border border-border/30 bg-muted/10 p-4"
			initial={{ opacity: 0, y: 12 }}
			animate={{ opacity: 1, y: 0 }}
			transition={{ duration: 0.35, delay: 0.2 }}
		>
			<h3 class="text-[var(--fs-small)] font-semibold text-foreground">Partisipasi 7 Hari</h3>
			<!-- Mini bar chart placeholder -->
			<div class="mt-3 flex items-end gap-1.5" style="height: 80px;">
				{#each [40, 65, 55, 80, 70, 90, 60] as height, i}
					<motion.div
						class="flex-1 rounded-t-sm bg-primary/20"
						initial={{ height: 0 }}
						animate={{ height: `${height}%` }}
						transition={{ duration: 0.4, delay: 0.25 + i * 0.05 }}
					>
						<div
							class="h-full w-full rounded-t-sm bg-primary/60"
							style="height: {Math.min(height + 10, 100)}%"
						></div>
					</motion.div>
				{/each}
			</div>
			<div class="mt-1.5 flex justify-between text-[10px] text-muted-foreground">
				<span>Sen</span><span>Sel</span><span>Rab</span><span>Kam</span><span>Jum</span><span>Sab</span><span>Min</span>
			</div>
		</motion.div>

		<!-- Recent activity feed -->
		<motion.div
			class="mt-4"
			initial={{ opacity: 0 }}
			animate={{ opacity: 1 }}
			transition={{ duration: 0.3, delay: 0.4 }}
		>
			<h3 class="text-[var(--fs-small)] font-semibold text-foreground">Aktivitas Terkini</h3>
			<div class="mt-2 space-y-2">
				{#each [
					{ icon: 'vouch', text: 'Pak Ahmad memberi Vouch pada laporan jalan rusak', time: '2m' },
					{ icon: 'contribute', text: 'Ibu Sari menyumbang Rp 500.000 untuk perbaikan', time: '15m' },
					{ icon: 'verify', text: '3 saksi baru bergabung untuk verifikasi banjir', time: '1j' },
				] as item}
					<div class="flex items-start gap-2.5 rounded-lg bg-muted/10 px-3 py-2">
						<div class="mt-0.5 size-2 shrink-0 rounded-full bg-primary/50"></div>
						<div class="min-w-0 flex-1">
							<p class="text-[var(--fs-caption)] leading-relaxed text-foreground/80">{item.text}</p>
							<p class="mt-0.5 text-[10px] text-muted-foreground">{item.time} lalu</p>
						</div>
					</div>
				{/each}
			</div>
		</motion.div>

		<!-- Tandang signal summary -->
		<motion.div
			class="mt-4 rounded-xl border border-border/30 bg-muted/10 p-4"
			initial={{ opacity: 0, y: 12 }}
			animate={{ opacity: 1, y: 0 }}
			transition={{ duration: 0.35, delay: 0.5 }}
		>
			<h3 class="text-[var(--fs-small)] font-semibold text-foreground">Sinyal Minggu Ini</h3>
			<div class="mt-3 space-y-2">
				{#each [
					{ label: 'Vouch', count: 45, color: 'bg-green-500', pct: 29 },
					{ label: 'Skeptis', count: 12, color: 'bg-amber-500', pct: 8 },
					{ label: 'Proof of Resolve', count: 38, color: 'bg-blue-500', pct: 24 },
					{ label: 'Bagus!', count: 42, color: 'bg-purple-500', pct: 27 },
					{ label: 'Perlu Dicek', count: 19, color: 'bg-red-400', pct: 12 },
				] as signal}
					<div class="flex items-center gap-2">
						<span class="w-24 text-[var(--fs-caption)] text-muted-foreground">{signal.label}</span>
						<div class="h-2 flex-1 rounded-full bg-muted/30">
							<div
								class="h-full rounded-full {signal.color} transition-all duration-500"
								style="width: {signal.pct}%"
							></div>
						</div>
						<span class="w-8 text-right text-[var(--fs-caption)] font-medium text-foreground">{signal.count}</span>
					</div>
				{/each}
			</div>
		</motion.div>
	</div>
</div>

<style>
	.stat-card {
		padding: 0.75rem;
		border-radius: var(--r-md);
		border: 1px solid oklch(from var(--c-batu) l c h / 0.3);
		background: oklch(from var(--c-susu) l c h / 0.5);
	}
</style>
