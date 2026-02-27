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

const triageShell = (page: import('@playwright/test').Page) =>
	page
		.locator('[data-testid="triage-shell"]')
		.filter({ has: page.locator('[data-testid="triage-opener"]:visible') })
		.first();

const triagePanel = (page: import('@playwright/test').Page) =>
	triageShell(page).getByTestId('triage-panel').first();

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

	const triageCard = triageShell(page).getByTestId('triage-opener');
	for (let attempt = 0; attempt < 2; attempt += 1) {
		await page.goto('/', { waitUntil: 'domcontentloaded' });
		try {
			await expect(triageCard).toBeVisible({ timeout: 15000 });
			await expect
				.poll(async () => triageShell(page).getAttribute('data-hydrated'), { timeout: 5000 })
				.toBe('true');
			break;
		} catch (error) {
			if (attempt === 1) throw error;
			await page.reload({ waitUntil: 'domcontentloaded' });
		}
	}

	await triageCard.evaluate((element) => (element as HTMLElement).click());
	await expect(triagePanel(page)).toHaveCount(1);
	await expect(triagePanel(page).getByTestId('triage-trajectory-grid')).toBeVisible({
		timeout: 5000
	});
}

async function submitTurn(page: import('@playwright/test').Page, text: string) {
	const panel = triagePanel(page);
	const textarea = panel.getByTestId('triage-input');
	const sendButton = panel.getByTestId('triage-send');
	const messageBubbles = panel.getByTestId('triage-message-bubble');
	const before = await messageBubbles.count();

	await expect(textarea).toBeEnabled({ timeout: 5000 });
	await textarea.fill(text);
	await expect(sendButton).toBeEnabled({ timeout: 5000 });
	await sendButton.click();
	await expect.poll(async () => messageBubbles.count(), { timeout: 10000 }).toBeGreaterThan(before);
}

async function completeTriageToReady(
	page: import('@playwright/test').Page,
	trajectoryId: 'masalah' | 'musyawarah' | 'catat',
	followUpPrefix: string
) {
	const primers: Record<typeof trajectoryId, string> = {
		masalah: 'Ada masalah jalan rusak di lingkungan kami.',
		musyawarah: 'Perlu musyawarah warga untuk keputusan taman bermain.',
		catat: 'Saya ingin catat data harga sembako minggu ini.'
	};
	await submitTurn(page, primers[trajectoryId]);
	await submitTurn(page, `${followUpPrefix} 1`);
	await submitTurn(page, `${followUpPrefix} 2`);
	await expect(triagePanel(page).getByTestId('triage-operator-blocks').first()).toBeVisible({
		timeout: 10000
	});
}

test.describe.skip('Triage Payload Render', () => {
	test.describe.configure({ mode: 'serial', timeout: 120_000 });

	test('renders structured and conversation payload for masalah flow', async ({
		context,
		page
	}) => {
		await authenticateAndOpenTriage(context, page);
		await completeTriageToReady(page, 'masalah', 'Tambah detail dampak');

		await expect(triagePanel(page).getByTestId('triage-structured-preview').first()).toBeVisible();
		await expect(triagePanel(page).locator('[data-slot="block-renderer"]').first()).toBeVisible();
		await expect(triagePanel(page).locator('[data-slot="card-envelope"]').first()).toBeVisible();
	});

	test('renders vote blocks for musyawarah flow', async ({ context, page }) => {
		await authenticateAndOpenTriage(context, page);
		await completeTriageToReady(page, 'musyawarah', 'Tambahkan poin mufakat');

		await expect(triagePanel(page).getByText('CHAT · Vote Card')).toBeVisible();
		await expect(triagePanel(page).locator('[data-slot="vote-block"]').first()).toBeVisible();
		await expect(triagePanel(page).locator('[data-card-type="vote_card"]').first()).toBeVisible();
	});

	test('renders form blocks for catat data flow', async ({ context, page }) => {
		await authenticateAndOpenTriage(context, page);
		await completeTriageToReady(page, 'catat', 'Lengkapi sumber data');

		await expect(triagePanel(page).getByTestId('triage-structured-preview').first()).toBeVisible();
		await expect(triagePanel(page).locator('[data-slot="form-block"]').first()).toBeVisible();
		await expect(triagePanel(page).locator('[data-slot="card-envelope"]').first()).toBeVisible();
	});
});
