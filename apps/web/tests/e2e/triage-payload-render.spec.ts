import { expect, test } from '@playwright/test';
import { SignJWT } from 'jose';

const DEFAULT_JWT_SECRET = 'playwright-jwt-secret';
const DEFAULT_SESSION_COOKIE_NAME = 'gr_session';

const createSessionToken = async () => {
	const secret = process.env.JWT_SECRET ?? DEFAULT_JWT_SECRET;
	const signingSecret = new TextEncoder().encode(secret);
	const now = Math.floor(Date.now() / 1000);

	return new SignJWT({
		sub: 'e2e-triage-payload-user',
		role: 'user',
		exp: now + 60 * 10
	})
		.setProtectedHeader({ alg: 'HS256' })
		.sign(signingSecret);
};

async function authenticateAndOpenTriage(
	context: import('@playwright/test').BrowserContext,
	page: import('@playwright/test').Page
) {
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

	const triageCard = page.getByRole('button', { name: /mulai di sini|start here/i });
	for (let attempt = 0; attempt < 2; attempt += 1) {
		await page.goto('/');
		try {
			await expect(triageCard).toBeVisible({ timeout: 15000 });
			break;
		} catch (error) {
			if (attempt === 1) throw error;
			await page.reload();
		}
	}

	await triageCard.evaluate((element) => (element as HTMLElement).click());
	await expect(page.locator('.triage-panel')).toBeVisible({ timeout: 5000 });
	await expect(page.locator('.trajectory-grid')).toBeVisible({ timeout: 5000 });
}

async function submitTurn(page: import('@playwright/test').Page, text: string) {
	const textarea = page.locator('.triage-panel textarea');
	await expect(textarea).toBeEnabled({ timeout: 5000 });
	await textarea.fill(text, { force: true });
	await textarea.press('Enter');
	await page.waitForTimeout(900);
}

async function completeTriageToReady(
	page: import('@playwright/test').Page,
	trajectoryLabel: string,
	followUpPrefix: string
) {
	const chip = page.locator('.triage-panel .trajectory-chip', { hasText: trajectoryLabel }).first();
	await chip.evaluate((element) => (element as HTMLButtonElement).click());
	await page.waitForTimeout(1000);
	await expect
		.poll(async () => page.locator('.triage-panel .rounded-2xl').count(), { timeout: 10000 })
		.toBeGreaterThan(0);
	await submitTurn(page, `${followUpPrefix} 1`);
	await submitTurn(page, `${followUpPrefix} 2`);
	await expect(page.getByText('Operator blocks')).toBeVisible({ timeout: 10000 });
}

test.describe.skip('Triage Payload Render', () => {
	test('renders structured and conversation payload for masalah flow', async ({
		context,
		page
	}) => {
		await authenticateAndOpenTriage(context, page);
		await completeTriageToReady(page, 'Masalah', 'Tambah detail dampak');

		await expect(page.locator('[data-slot="block-renderer"]').first()).toBeVisible();
		await expect(page.locator('[data-slot="card-envelope"]').first()).toBeVisible();
	});

	test('renders vote blocks for musyawarah flow', async ({ context, page }) => {
		await authenticateAndOpenTriage(context, page);
		await completeTriageToReady(page, 'Musyawarah', 'Tambahkan poin mufakat');

		await expect(page.getByText('CHAT · Vote Card')).toBeVisible();
		await expect(page.locator('[data-slot="vote-block"]').first()).toBeVisible();
		await expect(page.locator('[data-card-type="vote_card"]').first()).toBeVisible();
	});

	test('renders form blocks for catat data flow', async ({ context, page }) => {
		await authenticateAndOpenTriage(context, page);
		await completeTriageToReady(page, 'Catat', 'Lengkapi sumber data');

		await expect(page.locator('[data-slot="form-block"]').first()).toBeVisible();
		await expect(page.getByText('Kategori Data')).toBeVisible();
		await expect(page.locator('[data-slot="card-envelope"]').first()).toBeVisible();
	});
});
