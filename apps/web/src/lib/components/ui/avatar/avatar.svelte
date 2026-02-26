<script lang="ts" module>
	import { type VariantProps, tv } from 'tailwind-variants';

	export const avatarVariants = tv({
		base: 'relative flex shrink-0 overflow-hidden rounded-full bg-api-terang',
		variants: {
			size: {
				xs: 'size-5 text-[10px]',
				sm: 'size-7 text-xs',
				md: 'size-9 text-[13px]',
				lg: 'size-12 text-[17px]',
				xl: 'size-16 text-[22px]'
			}
		},
		defaultVariants: {
			size: 'md'
		}
	});

	export type AvatarSize = VariantProps<typeof avatarVariants>['size'];
</script>

<script lang="ts">
	import { Avatar as AvatarPrimitive } from 'bits-ui';
	import { cn } from '$lib/utils';

	let {
		ref = $bindable(null),
		loadingStatus = $bindable('loading'),
		class: className,
		size = 'md' as AvatarSize,
		...restProps
	}: AvatarPrimitive.RootProps & {
		size?: AvatarSize;
	} = $props();
</script>

<AvatarPrimitive.Root
	bind:ref
	bind:loadingStatus
	data-slot="avatar"
	class={cn(avatarVariants({ size }), className)}
	{...restProps}
/>
