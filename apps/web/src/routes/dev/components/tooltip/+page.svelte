<script lang="ts">
	import {
		Tooltip,
		TooltipContent,
		TooltipTrigger,
		TooltipProvider
	} from '$lib/components/ui/tooltip';
	import { Button } from '$lib/components/ui/button';
</script>

<h1 class="mb-6 text-2xl font-bold">Tooltip</h1>

<section class="mb-8">
	<h2 class="mb-4 text-lg font-semibold text-muted-foreground">Positions</h2>
	<div class="flex flex-wrap items-center gap-6">
		<TooltipProvider>
			<Tooltip>
				<TooltipTrigger>
					<Button variant="outline">Top</Button>
				</TooltipTrigger>
				<TooltipContent side="top" sideOffset={4}>Tooltip on top</TooltipContent>
			</Tooltip>
		</TooltipProvider>

		<TooltipProvider>
			<Tooltip>
				<TooltipTrigger>
					<Button variant="outline">Bottom</Button>
				</TooltipTrigger>
				<TooltipContent side="bottom" sideOffset={4}>Tooltip on bottom</TooltipContent>
			</Tooltip>
		</TooltipProvider>

		<TooltipProvider>
			<Tooltip>
				<TooltipTrigger>
					<Button variant="outline">Left</Button>
				</TooltipTrigger>
				<TooltipContent side="left" sideOffset={4}>Tooltip on left</TooltipContent>
			</Tooltip>
		</TooltipProvider>

		<TooltipProvider>
			<Tooltip>
				<TooltipTrigger>
					<Button variant="outline">Right</Button>
				</TooltipTrigger>
				<TooltipContent side="right" sideOffset={4}>Tooltip on right</TooltipContent>
			</Tooltip>
		</TooltipProvider>
	</div>
</section>

<section>
	<h2 class="mb-4 text-lg font-semibold text-muted-foreground">With DNA Buttons</h2>
	<div class="flex flex-wrap items-center gap-4">
		<TooltipProvider>
			<Tooltip>
				<TooltipTrigger>
					<Button variant="track-tuntaskan">Hover me</Button>
				</TooltipTrigger>
				<TooltipContent sideOffset={4}>Tuntaskan track action</TooltipContent>
			</Tooltip>
		</TooltipProvider>

		<TooltipProvider>
			<Tooltip>
				<TooltipTrigger>
					<Button variant="vault">Vault action</Button>
				</TooltipTrigger>
				<TooltipContent sideOffset={4}>Confidential vault action</TooltipContent>
			</Tooltip>
		</TooltipProvider>
	</div>
</section>
