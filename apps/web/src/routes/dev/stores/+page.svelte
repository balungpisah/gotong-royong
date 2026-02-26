<script lang="ts">
	import { dev } from '$app/environment';
	import { getWitnessStore, getUserStore, getNotificationStore, getTriageStore } from '$lib/stores';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';

	if (!dev) {
		throw new Error('Dev pages are not available in production');
	}

	const witnessStore = getWitnessStore();
	const userStore = getUserStore();
	const notificationStore = getNotificationStore();
	const triageStore = getTriageStore();
</script>

<div class="space-y-8">
	<div>
		<h1 class="text-h1 font-extrabold text-foreground">Stores</h1>
		<p class="text-body text-muted-foreground">
			Interactive state visualization for all Svelte 5 runes-based stores.
		</p>
	</div>

	<!-- User Store -->
	<section class="space-y-3">
		<h2 class="text-h2 font-bold text-foreground">User Store</h2>
		<div class="flex gap-2">
			<Button size="sm" onclick={() => userStore.loadCurrentUser()}>Load Current User</Button>
			<Button size="sm" variant="outline" onclick={() => userStore.logout()}>Logout</Button>
		</div>
		<div class="rounded-lg border border-border bg-card p-4">
			{#if userStore.loading}
				<p class="text-body text-muted-foreground">Memuat...</p>
			{:else if userStore.profile}
				<div class="space-y-1 text-body">
					<p><span class="font-medium">Nama:</span> {userStore.displayName}</p>
					<p>
						<span class="font-medium">Role:</span>
						<Badge variant="outline">{userStore.userRole}</Badge>
					</p>
					<p><span class="font-medium">Tier:</span> {userStore.userTier}</p>
					<p>
						<span class="font-medium">Authenticated:</span>
						{userStore.isAuthenticated ? 'Ya' : 'Tidak'}
					</p>
					<details class="mt-2">
						<summary class="cursor-pointer text-small text-muted-foreground">Raw JSON</summary>
						<pre class="mt-1 max-h-48 overflow-auto rounded bg-muted p-2 text-small">{JSON.stringify(
								userStore.profile,
								null,
								2
							)}</pre>
					</details>
				</div>
			{:else}
				<p class="text-body text-muted-foreground">Belum dimuat. Klik "Load Current User".</p>
			{/if}
			{#if userStore.error}
				<p class="mt-2 text-body text-destructive">{userStore.error}</p>
			{/if}
		</div>
	</section>

	<!-- Witness Store -->
	<section class="space-y-3">
		<h2 class="text-h2 font-bold text-foreground">Witness Store</h2>
		<div class="flex flex-wrap gap-2">
			<Button size="sm" onclick={() => witnessStore.loadList()}>Load Witnesses</Button>
			<Button size="sm" variant="outline" onclick={() => witnessStore.loadDetail('witness-001')}>
				Load Detail (witness-001)
			</Button>
			<Button
				size="sm"
				variant="outline"
				onclick={() => witnessStore.sendMessage('Pesan test dari dev gallery')}
				disabled={!witnessStore.current}
			>
				Send Message
			</Button>
		</div>
		<div class="grid gap-4 md:grid-cols-2">
			<!-- List -->
			<div class="rounded-lg border border-border bg-card p-4">
				<h3 class="mb-2 text-h3 font-semibold text-foreground">
					Witness List
					{#if witnessStore.listLoading}
						<Badge variant="outline" class="ml-2">Loading...</Badge>
					{/if}
				</h3>
				{#if witnessStore.witnesses.length > 0}
					<div class="space-y-2">
						{#each witnessStore.witnesses as w (w.witness_id)}
							<div
								class="flex items-center justify-between rounded border border-border/50 px-3 py-2 text-small"
							>
								<div>
									<p class="font-medium">{w.title}</p>
									<p class="text-muted-foreground">{w.status} · {w.track_hint}</p>
								</div>
								{#if w.unread_count > 0}
									<Badge variant="destructive" class="text-[10px]">{w.unread_count}</Badge>
								{/if}
							</div>
						{/each}
					</div>
					<div class="mt-3 flex gap-3 text-small text-muted-foreground">
						<span>Active: {witnessStore.activeWitnesses.length}</span>
						<span>Unread total: {witnessStore.unreadTotal}</span>
					</div>
				{:else}
					<p class="text-body text-muted-foreground">Belum dimuat.</p>
				{/if}
				{#if witnessStore.listError}
					<p class="mt-2 text-body text-destructive">{witnessStore.listError}</p>
				{/if}
			</div>

			<!-- Detail -->
			<div class="rounded-lg border border-border bg-card p-4">
				<h3 class="mb-2 text-h3 font-semibold text-foreground">
					Current Detail
					{#if witnessStore.detailLoading}
						<Badge variant="outline" class="ml-2">Loading...</Badge>
					{/if}
				</h3>
				{#if witnessStore.current}
					<div class="space-y-1 text-small">
						<p><span class="font-medium">ID:</span> {witnessStore.current.witness_id}</p>
						<p><span class="font-medium">Title:</span> {witnessStore.current.title}</p>
						<p><span class="font-medium">Messages:</span> {witnessStore.currentMessages.length}</p>
						<p>
							<span class="font-medium">Plan:</span>
							{witnessStore.currentPlan ? witnessStore.currentPlan.title : 'None'}
						</p>
						<p><span class="font-medium">Members:</span> {witnessStore.current.members.length}</p>
						<p><span class="font-medium">Blocks:</span> {witnessStore.current.blocks.length}</p>
					</div>
					<details class="mt-2">
						<summary class="cursor-pointer text-small text-muted-foreground">Messages JSON</summary>
						<pre class="mt-1 max-h-48 overflow-auto rounded bg-muted p-2 text-small">{JSON.stringify(
								witnessStore.currentMessages.map((m) => ({ id: m.message_id, type: m.type })),
								null,
								2
							)}</pre>
					</details>
				{:else}
					<p class="text-body text-muted-foreground">Belum dimuat.</p>
				{/if}
				{#if witnessStore.detailError}
					<p class="mt-2 text-body text-destructive">{witnessStore.detailError}</p>
				{/if}
			</div>
		</div>
	</section>

	<!-- Notification Store -->
	<section class="space-y-3">
		<h2 class="text-h2 font-bold text-foreground">Notification Store</h2>
		<div class="flex gap-2">
			<Button size="sm" onclick={() => notificationStore.loadNotifications()}
				>Load Notifications</Button
			>
			<Button size="sm" variant="outline" onclick={() => notificationStore.markAllRead()}>
				Mark All Read
			</Button>
		</div>
		<div class="rounded-lg border border-border bg-card p-4">
			{#if notificationStore.loading}
				<p class="text-body text-muted-foreground">Memuat...</p>
			{:else if notificationStore.notifications.length > 0}
				<div class="mb-3 flex gap-3 text-small text-muted-foreground">
					<span>Total: {notificationStore.notifications.length}</span>
					<span>
						Unread:
						<Badge
							variant={notificationStore.hasUnread ? 'destructive' : 'outline'}
							class="text-[10px]"
						>
							{notificationStore.unreadCount}
						</Badge>
					</span>
				</div>
				<div class="space-y-1">
					{#each notificationStore.notifications as n (n.notification_id)}
						<div
							class="flex items-center justify-between rounded px-3 py-1.5 text-small {n.read
								? 'text-muted-foreground'
								: 'bg-muted/50 font-medium text-foreground'}"
						>
							<div>
								<span class="mr-2 text-[10px] uppercase text-muted-foreground">{n.type}</span>
								{n.title}
							</div>
							{#if !n.read}
								<Button
									size="sm"
									variant="ghost"
									class="h-6 px-2 text-[10px]"
									onclick={() => notificationStore.markRead(n.notification_id)}
								>
									Read
								</Button>
							{/if}
						</div>
					{/each}
				</div>
			{:else}
				<p class="text-body text-muted-foreground">Belum dimuat.</p>
			{/if}
			{#if notificationStore.error}
				<p class="mt-2 text-body text-destructive">{notificationStore.error}</p>
			{/if}
		</div>
	</section>

	<!-- Triage Store -->
	<section class="space-y-3">
		<h2 class="text-h2 font-bold text-foreground">Triage Store</h2>
		<div class="flex gap-2">
			<Button
				size="sm"
				onclick={() => triageStore.startTriage('Jalan di depan rumah saya rusak parah')}
			>
				Start Triage
			</Button>
			<Button
				size="sm"
				variant="outline"
				onclick={() => triageStore.updateTriage('Sudah 3 bulan tidak diperbaiki')}
				disabled={!triageStore.result}
			>
				Update Triage
			</Button>
			<Button size="sm" variant="outline" onclick={() => triageStore.reset()}>Reset</Button>
		</div>
		<div class="rounded-lg border border-border bg-card p-4">
			{#if triageStore.loading}
				<p class="text-body text-muted-foreground">Memuat...</p>
			{:else if triageStore.result}
				<div class="space-y-1 text-body">
					<p>
						<span class="font-medium">Bar State:</span>
						<Badge variant="outline">{triageStore.barState}</Badge>
					</p>
					<p><span class="font-medium">Is Ready:</span> {triageStore.isReady ? 'Ya' : 'Belum'}</p>
					{#if triageStore.confidence}
						<p><span class="font-medium">Confidence:</span> {triageStore.confidence.label}</p>
					{/if}
					{#if triageStore.trackHint}
						<p><span class="font-medium">Track:</span> {triageStore.trackHint}</p>
					{/if}
					{#if triageStore.proposedPlan}
						<p><span class="font-medium">Plan:</span> {triageStore.proposedPlan.title}</p>
					{/if}
				</div>
			{:else}
				<p class="text-body text-muted-foreground">Belum dimulai. Klik "Start Triage".</p>
			{/if}
			{#if triageStore.error}
				<p class="mt-2 text-body text-destructive">{triageStore.error}</p>
			{/if}
		</div>
	</section>
</div>
