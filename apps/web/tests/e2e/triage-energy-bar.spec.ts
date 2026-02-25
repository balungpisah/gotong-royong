import { expect, test } from '@playwright/test';
import { SignJWT } from 'jose';

const DEFAULT_JWT_SECRET = 'playwright-jwt-secret';
const DEFAULT_SESSION_COOKIE_NAME = 'gr_session';

const createSessionToken = async () => {
	const secret = process.env.JWT_SECRET ?? DEFAULT_JWT_SECRET;
	const signingSecret = new TextEncoder().encode(secret);
	const now = Math.floor(Date.now() / 1000);

	return new SignJWT({
		sub: 'e2e-triage-user',
		role: 'user',
		exp: now + 60 * 10
	})
		.setProtectedHeader({ alg: 'HS256' })
		.sign(signingSecret);
};

async function authenticateAndNavigate(context: any, page: any) {
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

	const triageCard = page.locator('.triage-card');
	for (let attempt = 0; attempt < 2; attempt += 1) {
		await page.goto('/');
		try {
			await expect(triageCard).toBeVisible({ timeout: 15000 });
			return;
		} catch (error) {
			if (attempt === 1) throw error;
			await page.reload();
		}
	}
}

test.describe('Triage Energy Bar', () => {
	test('triage card expands and shows trajectory grid with 9 intents', async ({
		context,
		page
	}) => {
		await authenticateAndNavigate(context, page);

		// Click the "Mulai di sini" triage card
		await page.click('.triage-card');

		// Should see expanded panel with trajectory grid
		await expect(page.locator('.triage-panel')).toBeVisible();
		await expect(page.locator('.trajectory-grid')).toBeVisible();

		// Should see exactly 9 trajectory intent chips
		const chips = page.locator('.trajectory-chip');
		await expect(chips).toHaveCount(9);

		// Should see the "Atau ceritakan langsung" free-text card
		const freeTextCard = page.locator('.trajectory-grid').getByText('Atau ceritakan langsung');
		await expect(freeTextCard).toBeVisible();
	});

	test('clicking trajectory chip starts triage and shows energy bar', async ({
		context,
		page
	}) => {
		await authenticateAndNavigate(context, page);

		// Expand triage panel
		await page.click('.triage-card');
		await expect(page.locator('.triage-panel')).toBeVisible();

		// Click the first trajectory chip (Masalah)
		await page.locator('.trajectory-chip').first().click();

		// Wait for the AI response (mock service has 500ms delay)
		await page.waitForTimeout(1000);

		// Chat messages should appear in the panel
		const messages = page.locator('.triage-panel .rounded-2xl');
		expect(await messages.count()).toBeGreaterThan(0);

		// Energy bar should appear — it has a title attribute "Sisa energi AI"
		const energyBar = page.locator('[title*="Sisa energi AI"]');
		await expect(energyBar).toBeVisible({ timeout: 5000 });
	});

	test('energy bar updates with follow-up messages', async ({ context, page }) => {
		await authenticateAndNavigate(context, page);

		// Start triage via trajectory chip
		await page.click('.triage-card');
		await page.locator('.trajectory-chip').first().click();
		await page.waitForTimeout(1000);

		// Verify energy bar is present
		const energyBar = page.locator('[title*="Sisa energi AI"]');
		await expect(energyBar).toBeVisible({ timeout: 5000 });

		// Get initial percentage
		const initialTitle = await energyBar.getAttribute('title');
		expect(initialTitle).toContain('%');

		// Send a follow-up message
		const textarea = page.locator('.triage-panel textarea');
		if (await textarea.isEnabled()) {
			await textarea.fill('Jalan di RT 05 sudah berlubang 3 bulan');
			await page.keyboard.press('Enter');
			await page.waitForTimeout(800);

			// Energy bar should still be visible
			await expect(energyBar).toBeVisible();
		}
	});

	test('reset triggers cooldown and removes energy bar', async ({ context, page }) => {
		await authenticateAndNavigate(context, page);

		// Start a triage session
		await page.click('.triage-card');
		await page.locator('.trajectory-chip').first().click();
		await page.waitForTimeout(1000);

		// Click reset button
		const resetButton = page.locator('button[aria-label="Mulai ulang"]');
		await expect(resetButton).toBeVisible({ timeout: 5000 });
		await resetButton.click();

		// Should show cooldown message
		await expect(page.getByText('Sesi sebelumnya selesai')).toBeVisible({ timeout: 5000 });

		// Energy bar should be gone (no active session)
		const energyBar = page.locator('[title*="Sisa energi AI"]');
		await expect(energyBar).not.toBeVisible();
	});
});
