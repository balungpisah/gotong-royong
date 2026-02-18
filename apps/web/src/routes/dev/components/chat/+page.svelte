<script lang="ts">
	import { ChatThread } from '$lib/components/chat';
	import { mockChatMessages } from '$lib/fixtures/mock-chat';
</script>

<div class="flex flex-col gap-6">
	<div>
		<h1 class="text-2xl font-bold">Chat Thread</h1>
		<p class="mt-1 text-sm text-muted-foreground">
			All 7 chat message types rendered in a WhatsApp-style thread
		</p>
	</div>

	<!-- Chat container with mobile-width simulation -->
	<div class="mx-auto w-full max-w-md rounded-lg border border-border bg-background p-4 shadow-sm">
		<ChatThread messages={mockChatMessages} />
	</div>

	<!-- Legend -->
	<div class="rounded-lg border border-border bg-card p-4">
		<h3 class="mb-2 text-sm font-bold">Message Types in This Thread</h3>
		<div class="grid grid-cols-2 gap-2 text-xs text-muted-foreground sm:grid-cols-3">
			<span>1. System Message (member_joined)</span>
			<span>2. User Message (self)</span>
			<span>3. AI Card (classification)</span>
			<span>4. User Message (other)</span>
			<span>5. Evidence Card</span>
			<span>6. Diff Card</span>
			<span>7. Vote Card</span>
			<span>8. System Message (phase)</span>
			<span>9. Galang Message</span>
			<span>10. AI Card (summary)</span>
		</div>
	</div>
</div>
