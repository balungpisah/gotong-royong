import { expect, test } from '@playwright/test';
import { SignJWT } from 'jose';

const DEFAULT_JWT_SECRET = 'playwright-jwt-secret';
const DEFAULT_SESSION_COOKIE_NAME = 'gr_session';

const createSessionToken = async () => {
	const secret = process.env.JWT_SECRET ?? DEFAULT_JWT_SECRET;
	const signingSecret = new TextEncoder().encode(secret);
	const now = Math.floor(Date.now() / 1000);

	return new SignJWT({
		sub: 'e2e-profile-user',
		role: 'user',
		exp: now + 60 * 10
	})
		.setProtectedHeader({ alg: 'HS256' })
		.sign(signingSecret);
};

test('non-self profile page fetches API profile endpoint', async ({ context, page }) => {
	const token = await createSessionToken();
	const baseURL = test.info().project.use.baseURL;
	if (!baseURL) throw new Error('Playwright baseURL is required');

	await context.addCookies([
		{
			name: process.env.GR_SESSION_COOKIE_NAME ?? DEFAULT_SESSION_COOKIE_NAME,
			value: token,
			url: `${baseURL}/`
		}
	]);

	let profileRequestHits = 0;
	await page.route('**/v1/tandang/users/user-999/profile', async (route) => {
		profileRequestHits += 1;
		await route.fulfill({
			status: 200,
			contentType: 'application/json',
			body: JSON.stringify({
				data: {
					platform_user_id: 'user-999',
					identity: 'gotong_royong:user-999',
					reputation: { username: 'Budi', percentile: 44 },
					tier: { tier_symbol: '◆◆◇◇' },
					activity: { last_active_at: '2026-02-25T00:00:00.000Z' },
					cv_hidup: {
						community_id: 'rw-09',
						community_name: 'RW 09',
						joined_at: '2026-02-01T00:00:00.000Z'
					}
				}
			})
		});
	});

	await page.goto('/profil/user-999');

	await expect(page).toHaveURL(/\/profil\/user-999$/);
	await expect(page.getByText('Budi')).toBeVisible();
	await expect.poll(() => profileRequestHits).toBe(1);
});

test('feed avatar deep-link opens non-self profile via API endpoint', async ({ context, page }) => {
	const token = await createSessionToken();
	const baseURL = test.info().project.use.baseURL;
	if (!baseURL) throw new Error('Playwright baseURL is required');

	await context.addCookies([
		{
			name: process.env.GR_SESSION_COOKIE_NAME ?? DEFAULT_SESSION_COOKIE_NAME,
			value: token,
			url: `${baseURL}/`
		}
	]);

	let profileRequestHits = 0;
	let requestedUserId: string | null = null;
	await page.route('**/v1/tandang/users/*/profile', async (route) => {
		profileRequestHits += 1;
		const requestUrl = new URL(route.request().url());
		const matched = requestUrl.pathname.match(/\/v1\/tandang\/users\/([^/]+)\/profile$/);
		requestedUserId = matched?.[1] ?? 'unknown-user';
		await route.fulfill({
			status: 200,
			contentType: 'application/json',
			body: JSON.stringify({
				data: {
					platform_user_id: requestedUserId,
					identity: `gotong_royong:${requestedUserId}`,
					reputation: { username: `Warga ${requestedUserId}`, percentile: 51 },
					tier: { tier_symbol: '◆◆◇◇' },
					activity: { last_active_at: '2026-02-25T00:00:00.000Z' },
					cv_hidup: {
						community_id: 'rw-09',
						community_name: 'RW 09',
						joined_at: '2026-02-01T00:00:00.000Z'
					}
				}
			})
		});
	});

	await page.goto('/');

	const firstProfileLink = page.locator('[data-profile-link]').first();
	await expect(firstProfileLink).toBeVisible({ timeout: 15000 });

	const targetUserId = await firstProfileLink.getAttribute('data-profile-user-id');
	if (!targetUserId) throw new Error('Missing profile deep-link user id');

	await firstProfileLink.click();

	await expect(page).toHaveURL(new RegExp(`/profil/${targetUserId}$`));
	await expect.poll(() => profileRequestHits).toBe(1);
	await expect.poll(() => requestedUserId).toBe(targetUserId);
	await expect(page.getByText(`Warga ${targetUserId}`)).toBeVisible();
});
