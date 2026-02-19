/**
 * Share utility — Web Share API with clipboard fallback.
 *
 * On mobile: triggers the native OS share sheet (WhatsApp, Telegram, etc.)
 * On desktop: copies the share URL to clipboard as fallback.
 *
 * The share text is crafted for civic engagement virality:
 * hook_line + title + witness URL → makes people curious and click.
 */

import type { FeedItem } from '$lib/types';

/** Base URL for witness deep links. TODO: Replace with env var when deployed. */
const BASE_URL = 'https://gotongroyong.id';

/** Build the shareable URL for a witness. */
export function getWitnessUrl(witnessId: string): string {
	return `${BASE_URL}/saksi/${witnessId}`;
}

/** Build share text from a feed item. */
export function buildShareText(item: FeedItem): { title: string; text: string; url: string } {
	const url = getWitnessUrl(item.witness_id);

	// Use hook_line if available (punchier), otherwise fall back to latest event snippet
	const hook = item.hook_line ?? item.latest_event.snippet ?? '';
	const memberInfo = item.member_count > 1 ? ` — ${item.member_count} warga terlibat` : '';

	return {
		title: item.title,
		text: `${hook}\n${item.title}${memberInfo}`,
		url
	};
}

/**
 * Share a feed item using the best available method.
 *
 * 1. Web Share API (mobile) → native share sheet with all installed apps
 * 2. Clipboard fallback (desktop) → copies URL, returns 'copied'
 *
 * @returns 'shared' if Web Share API succeeded,
 *          'copied' if fell back to clipboard,
 *          'dismissed' if user cancelled the share sheet,
 *          'error' if something went wrong.
 */
export async function shareFeedItem(
	item: FeedItem
): Promise<'shared' | 'copied' | 'dismissed' | 'error'> {
	const { title, text, url } = buildShareText(item);

	// Try Web Share API first (supported on most mobile browsers)
	if (typeof navigator !== 'undefined' && navigator.share) {
		try {
			await navigator.share({ title, text, url });
			return 'shared';
		} catch (err) {
			// User cancelled the share sheet — not an error
			if (err instanceof Error && err.name === 'AbortError') {
				return 'dismissed';
			}
			// Fall through to clipboard
		}
	}

	// Fallback: copy URL to clipboard
	try {
		if (typeof navigator !== 'undefined' && navigator.clipboard) {
			await navigator.clipboard.writeText(`${text}\n${url}`);
			return 'copied';
		}
	} catch {
		// Clipboard API not available or permission denied
	}

	return 'error';
}
