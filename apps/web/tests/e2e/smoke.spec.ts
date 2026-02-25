import { expect, test } from '@playwright/test';
import { SignJWT } from 'jose';

const DEFAULT_JWT_SECRET = 'playwright-jwt-secret';
const DEFAULT_SESSION_COOKIE_NAME = 'gr_session';

const createSessionToken = async () => {
	const secret = process.env.JWT_SECRET ?? DEFAULT_JWT_SECRET;
	const signingSecret = new TextEncoder().encode(secret);
	const now = Math.floor(Date.now() / 1000);

	return new SignJWT({
		sub: 'e2e-smoke-user',
		role: 'user',
		exp: now + 60 * 10
	})
		.setProtectedHeader({ alg: 'HS256' })
		.sign(signingSecret);
};

async function gotoHomeAndWaitForShell(page: import('@playwright/test').Page) {
	const brandLink = page.getByRole('link', { name: 'Gotong Royong' });
	const triageCard = page.locator('.triage-card');
	for (let attempt = 0; attempt < 2; attempt += 1) {
		await page.goto('/');
		await expect(page).toHaveURL(/\/$/);
		try {
			await expect(brandLink).toBeVisible({ timeout: 10000 });
			await expect(triageCard).toBeVisible({ timeout: 15000 });
			return;
		} catch (error) {
			if (attempt === 1) throw error;
			await page.reload();
		}
	}
}

test('guest is redirected from protected route to login with no-store headers', async ({
	page
}) => {
	const response = await page.request.get('/', { maxRedirects: 0 });

	expect(response.status()).toBe(303);
	expect(response.headers()['location']).toContain('/masuk');
	expect(response.headers()['cache-control']).toContain('no-store');

	await page.goto('/');

	await expect(page).toHaveURL(/\/masuk$/);
	await expect(page.getByRole('button', { name: 'Masuk ke Gotong Royong' })).toBeVisible();
});

test('authenticated user can access app shell', async ({ context, page }) => {
	const token = await createSessionToken();
	const baseURL = test.info().project.use.baseURL;

	if (!baseURL) {
		throw new Error('Playwright baseURL is required for cookie injection.');
	}

	await context.addCookies([
		{
			name: process.env.GR_SESSION_COOKIE_NAME ?? DEFAULT_SESSION_COOKIE_NAME,
			value: token,
			url: `${baseURL}/`
		}
	]);

	await gotoHomeAndWaitForShell(page);
});
