<script lang="ts">
	import { Progress, ProgressConfidenceBar } from '$lib/components/ui/progress';
</script>

<h1 class="mb-6 text-h1 font-extrabold">Progress</h1>

<section class="mb-8">
	<h2 class="mb-4 text-h2 font-bold text-muted-foreground">Default Progress</h2>
	<div class="max-w-md space-y-4">
		<div class="space-y-1">
			<span class="text-small text-muted-foreground">0%</span>
			<Progress value={0} />
		</div>
		<div class="space-y-1">
			<span class="text-small text-muted-foreground">25%</span>
			<Progress value={25} />
		</div>
		<div class="space-y-1">
			<span class="text-small text-muted-foreground">50%</span>
			<Progress value={50} />
		</div>
		<div class="space-y-1">
			<span class="text-small text-muted-foreground">75%</span>
			<Progress value={75} />
		</div>
		<div class="space-y-1">
			<span class="text-small text-muted-foreground">100%</span>
			<Progress value={100} />
		</div>
	</div>
</section>

<section>
	<h2 class="mb-4 text-h2 font-bold text-muted-foreground">Confidence Bar</h2>
	<div class="max-w-md space-y-4">
		<div class="space-y-1">
			<span class="text-small text-muted-foreground">High (85%)</span>
			<ProgressConfidenceBar value={85} />
		</div>
		<div class="space-y-1">
			<span class="text-small text-muted-foreground">Medium (55%)</span>
			<ProgressConfidenceBar value={55} />
		</div>
		<div class="space-y-1">
			<span class="text-small text-muted-foreground">Low (20%)</span>
			<ProgressConfidenceBar value={20} />
		</div>
		<div class="space-y-1">
			<span class="text-small text-muted-foreground">No label</span>
			<ProgressConfidenceBar value={42} showLabel={false} />
		</div>
	</div>
</section>
