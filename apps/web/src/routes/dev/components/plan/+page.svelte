<script lang="ts">
	import { PhaseBreadcrumb, PhaseCard, PathPlanView, DrawablePanel } from '$lib/components/blocks';
	import { Button } from '$lib/components/ui/button';
	import { mockPathPlan } from '$lib/fixtures/mock-plan';
</script>

<div class="flex flex-col gap-8">
	<div>
		<h1 class="text-h1 font-extrabold">Plan & Phase Components</h1>
		<p class="mt-1 text-body text-muted-foreground">
			Phase breadcrumb, phase cards, path plan view, and drawable panel
		</p>
	</div>

	<!-- Phase Breadcrumb (standalone) -->
	<section class="flex flex-col gap-3">
		<h2 class="border-b border-border pb-2 text-h2 font-bold">Phase Breadcrumb</h2>
		<div class="rounded-lg border border-border bg-card p-4">
			<PhaseBreadcrumb phases={mockPathPlan.branches[0].phases} />
		</div>
	</section>

	<!-- Single Phase Card -->
	<section class="flex flex-col gap-3">
		<h2 class="border-b border-border pb-2 text-h2 font-bold">Phase Card (single)</h2>
		<PhaseCard phase={mockPathPlan.branches[0].phases[1]} />
	</section>

	<!-- Full Path Plan View -->
	<section class="flex flex-col gap-3">
		<h2 class="border-b border-border pb-2 text-h2 font-bold">Path Plan View (full)</h2>
		<div class="rounded-lg border border-border bg-background p-4">
			<PathPlanView plan={mockPathPlan} />
		</div>
	</section>

	<!-- Drawable Panel -->
	<section class="flex flex-col gap-3">
		<h2 class="border-b border-border pb-2 text-h2 font-bold">Drawable Panel (Sheet)</h2>
		<p class="text-body text-muted-foreground">Click the button to open the slide-over panel</p>
		<DrawablePanel plan={mockPathPlan}>
			{#snippet trigger()}
				<Button variant="outline">Buka Rencana →</Button>
			{/snippet}
		</DrawablePanel>
	</section>
</div>
