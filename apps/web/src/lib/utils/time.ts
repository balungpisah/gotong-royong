/**
 * Time formatting utilities (Indonesian locale).
 */

/**
 * Formats an ISO timestamp as a relative time string in Indonesian.
 * e.g. "baru saja", "5 menit lalu", "2 jam lalu", "3 hari lalu"
 */
export function timeAgo(iso: string): string {
	const diff = Date.now() - new Date(iso).getTime();
	const mins = Math.floor(diff / 60_000);
	if (mins < 1) return 'baru saja';
	if (mins < 60) return `${mins} menit lalu`;
	const hours = Math.floor(mins / 60);
	if (hours < 24) return `${hours} jam lalu`;
	const days = Math.floor(hours / 24);
	if (days < 7) return `${days} hari lalu`;
	return `${Math.floor(days / 7)} minggu lalu`;
}
