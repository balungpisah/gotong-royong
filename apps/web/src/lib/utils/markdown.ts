/**
 * Markdown Utility — renders markdown to sanitized HTML.
 *
 * Uses `marked` for GFM parsing and `DOMPurify` for XSS sanitization.
 * Output is safe for {@html} in Svelte templates.
 */

import { Marked } from 'marked';
import DOMPurify from 'dompurify';

const marked = new Marked({
	gfm: true,
	breaks: true
});

/**
 * Render a markdown string to sanitized HTML.
 * Safe for use with Svelte's {@html} directive.
 */
export function renderMarkdown(md: string): string {
	const raw = marked.parse(md);
	if (typeof raw !== 'string') {
		// marked.parse can return Promise if async extensions are used;
		// we only use sync, so this is a safety fallback.
		return '';
	}
	return DOMPurify.sanitize(raw);
}
