<script lang="ts">
	/**
	 * DynamicIcon — resolves a Lucide icon name string to a rendered component.
	 *
	 * Since Svelte can't do truly dynamic imports of Lucide icons at runtime
	 * (tree-shaking breaks), we use a curated registry of ~50 icons covering
	 * common community case content. The LLM picks from these during Card
	 * Enrichment. Unknown names fall back to trajectory default → HelpCircle.
	 *
	 * ~50 Lucide icons add ~15KB gzipped (tiny).
	 *
	 * @see docs/design/specs/ai-spec/04b-trajectory-map.md
	 */

	import type { Component } from 'svelte';

	// ── Infrastructure & Public Services ─────────────────────────────
	import Construction from '@lucide/svelte/icons/construction';
	import Building2 from '@lucide/svelte/icons/building-2';
	import Landmark from '@lucide/svelte/icons/landmark';
	import House from '@lucide/svelte/icons/house';
	import TrafficCone from '@lucide/svelte/icons/traffic-cone';

	// ── Nature & Environment ─────────────────────────────────────────
	import TreePine from '@lucide/svelte/icons/tree-pine';
	import Droplets from '@lucide/svelte/icons/droplets';
	import Leaf from '@lucide/svelte/icons/leaf';
	import Sun from '@lucide/svelte/icons/sun';
	import CloudRain from '@lucide/svelte/icons/cloud-rain';
	import Mountain from '@lucide/svelte/icons/mountain';
	import Waves from '@lucide/svelte/icons/waves';

	// ── Legal & Governance ───────────────────────────────────────────
	import Scale from '@lucide/svelte/icons/scale';
	import Gavel from '@lucide/svelte/icons/gavel';
	import FileText from '@lucide/svelte/icons/file-text';
	import ClipboardList from '@lucide/svelte/icons/clipboard-list';
	import ScrollText from '@lucide/svelte/icons/scroll-text';
	import Shield from '@lucide/svelte/icons/shield';

	// ── Community & People ───────────────────────────────────────────
	import Users from '@lucide/svelte/icons/users';
	import Handshake from '@lucide/svelte/icons/handshake';
	import Heart from '@lucide/svelte/icons/heart';
	import Baby from '@lucide/svelte/icons/baby';
	import Megaphone from '@lucide/svelte/icons/megaphone';
	import MessageCircle from '@lucide/svelte/icons/message-circle';

	// ── Health & Safety ──────────────────────────────────────────────
	import HeartPulse from '@lucide/svelte/icons/heart-pulse';
	import Siren from '@lucide/svelte/icons/siren';
	import Flame from '@lucide/svelte/icons/flame';
	import ShieldAlert from '@lucide/svelte/icons/shield-alert';
	import AlertTriangle from '@lucide/svelte/icons/alert-triangle';
	import CircleAlert from '@lucide/svelte/icons/circle-alert';

	// ── Economy & Trade ──────────────────────────────────────────────
	import Banknote from '@lucide/svelte/icons/banknote';
	import ShoppingCart from '@lucide/svelte/icons/shopping-cart';
	import Store from '@lucide/svelte/icons/store';
	import TrendingUp from '@lucide/svelte/icons/trending-up';
	import Receipt from '@lucide/svelte/icons/receipt';

	// ── Education & Knowledge ────────────────────────────────────────
	import BookOpen from '@lucide/svelte/icons/book-open';
	import GraduationCap from '@lucide/svelte/icons/graduation-cap';
	import Lightbulb from '@lucide/svelte/icons/lightbulb';

	// ── Transport & Movement ─────────────────────────────────────────
	import Car from '@lucide/svelte/icons/car';
	import Bike from '@lucide/svelte/icons/bike';
	import MapPin from '@lucide/svelte/icons/map-pin';
	import Route from '@lucide/svelte/icons/route';

	// ── Celebration & Achievement ────────────────────────────────────
	import PartyPopper from '@lucide/svelte/icons/party-popper';
	import Trophy from '@lucide/svelte/icons/trophy';
	import Star from '@lucide/svelte/icons/star';
	import Medal from '@lucide/svelte/icons/medal';

	// ── Data & Monitoring ────────────────────────────────────────────
	import Eye from '@lucide/svelte/icons/eye';
	import BarChart3 from '@lucide/svelte/icons/bar-chart-3';
	import Lock from '@lucide/svelte/icons/lock';
	import Clock from '@lucide/svelte/icons/clock';
	import Calendar from '@lucide/svelte/icons/calendar';

	// ── Management ──────────────────────────────────────────────────
	import Settings from '@lucide/svelte/icons/settings';

	// ── Fallback ─────────────────────────────────────────────────────
	import HelpCircle from '@lucide/svelte/icons/help-circle';

	// ── Registry ─────────────────────────────────────────────────────

	const ICON_REGISTRY: Record<string, Component<{ class?: string }>> = {
		// Infrastructure & Public Services
		'construction': Construction,
		'building-2': Building2,
		'landmark': Landmark,
		'house': House,
		'traffic-cone': TrafficCone,

		// Nature & Environment
		'tree-pine': TreePine,
		'droplets': Droplets,
		'leaf': Leaf,
		'sun': Sun,
		'cloud-rain': CloudRain,
		'mountain': Mountain,
		'waves': Waves,

		// Legal & Governance
		'scale': Scale,
		'gavel': Gavel,
		'file-text': FileText,
		'clipboard-list': ClipboardList,
		'scroll-text': ScrollText,
		'shield': Shield,

		// Community & People
		'users': Users,
		'handshake': Handshake,
		'heart': Heart,
		'baby': Baby,
		'megaphone': Megaphone,
		'message-circle': MessageCircle,

		// Health & Safety
		'heart-pulse': HeartPulse,
		'siren': Siren,
		'flame': Flame,
		'shield-alert': ShieldAlert,
		'alert-triangle': AlertTriangle,
		'circle-alert': CircleAlert,

		// Economy & Trade
		'banknote': Banknote,
		'shopping-cart': ShoppingCart,
		'store': Store,
		'trending-up': TrendingUp,
		'receipt': Receipt,

		// Education & Knowledge
		'book-open': BookOpen,
		'graduation-cap': GraduationCap,
		'lightbulb': Lightbulb,

		// Transport & Movement
		'car': Car,
		'bike': Bike,
		'map-pin': MapPin,
		'route': Route,

		// Celebration & Achievement
		'party-popper': PartyPopper,
		'trophy': Trophy,
		'star': Star,
		'medal': Medal,

		// Data & Monitoring
		'eye': Eye,
		'bar-chart-3': BarChart3,
		'lock': Lock,
		'clock': Clock,
		'calendar': Calendar,

		// Management
		'settings': Settings,

		// Fallback
		'help-circle': HelpCircle
	};

	/**
	 * Default icon per trajectory type — used when the AI-selected icon
	 * name doesn't match anything in the registry.
	 */
	const TRAJECTORY_DEFAULT_ICON: Record<string, string> = {
		aksi: 'construction',
		advokasi: 'megaphone',
		pantau: 'eye',
		mufakat: 'users',
		mediasi: 'handshake',
		program: 'calendar',
		data: 'bar-chart-3',
		vault: 'lock',
		bantuan: 'heart',
		pencapaian: 'trophy',
		siaga: 'siren'
	};

	// ── Props ────────────────────────────────────────────────────────

	interface Props {
		/** Lucide icon name in kebab-case (e.g., "construction", "tree-pine"). */
		name: string;
		/** CSS classes to pass to the icon component. */
		class?: string;
		/** Trajectory type — used for fallback icon when name doesn't match. */
		fallback?: string;
	}

	let { name, class: className = '', fallback }: Props = $props();

	// ── Resolution ───────────────────────────────────────────────────

	function resolveIcon(iconName: string, trajectoryFallback?: string): Component<{ class?: string }> {
		// 1. Try exact name match
		if (ICON_REGISTRY[iconName]) return ICON_REGISTRY[iconName];

		// 2. Try trajectory-type default
		if (trajectoryFallback) {
			const defaultName = TRAJECTORY_DEFAULT_ICON[trajectoryFallback];
			if (defaultName && ICON_REGISTRY[defaultName]) return ICON_REGISTRY[defaultName];
		}

		// 3. Ultimate fallback
		return HelpCircle;
	}

	const IconComponent = $derived(resolveIcon(name, fallback));
</script>

<IconComponent class={className} />
