import { expect, test } from '@playwright/test';

const HARNESS_ROUTE = '/dev/e2e/triage';

const triageShell = (page: import('@playwright/test').Page) =>
	page.getByTestId('e2e-triage-harness').getByTestId('triage-shell').first();

const triagePanel = (page: import('@playwright/test').Page) =>
	triageShell(page).getByTestId('triage-panel').first();

async function openHarness(page: import('@playwright/test').Page) {
	for (let attempt = 0; attempt < 2; attempt += 1) {
		await page.goto(HARNESS_ROUTE, { waitUntil: 'domcontentloaded' });
		try {
			await expect(page.getByTestId('e2e-triage-harness')).toBeVisible({ timeout: 15_000 });
			await expect(triageShell(page).getByTestId('triage-opener')).toBeVisible({
				timeout: 10_000
			});
			await expect
				.poll(async () => triageShell(page).getAttribute('data-hydrated'), {
					timeout: 5_000
				})
				.toBe('true');
			return;
		} catch (error) {
			if (attempt === 1) throw error;
			await page.reload({ waitUntil: 'domcontentloaded' });
		}
	}
}

async function authenticateAndOpenTriage(page: import('@playwright/test').Page) {
	await openHarness(page);
	await triageShell(page)
		.getByTestId('triage-opener')
		.evaluate((element) => (element as HTMLElement).click());
	await expect(triagePanel(page)).toHaveCount(1);
	await expect(triagePanel(page).getByTestId('triage-trajectory-grid')).toBeVisible({
		timeout: 5_000
	});
}

async function submitTurn(page: import('@playwright/test').Page, text: string) {
	const panel = triagePanel(page);
	const textarea = panel.getByTestId('triage-input');
	const sendButton = panel.getByTestId('triage-send');
	const messageBubbles = panel.getByTestId('triage-message-bubble');
	const before = await messageBubbles.count();

	await expect(textarea).toBeEnabled({ timeout: 5_000 });
	await textarea.fill(text);
	await expect(sendButton).toBeEnabled({ timeout: 5_000 });
	await sendButton.click();
	await expect
		.poll(async () => messageBubbles.count(), { timeout: 10_000 })
		.toBeGreaterThan(before);
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
		timeout: 10_000
	});
}

test.describe('Triage Payload Render', () => {
	test.describe.configure({ mode: 'serial', timeout: 120_000 });

	test('renders structured and conversation payload for masalah flow', async ({ page }) => {
		await authenticateAndOpenTriage(page);
		await completeTriageToReady(page, 'masalah', 'Tambah detail dampak');

		await expect(triagePanel(page).getByTestId('triage-structured-preview').first()).toBeVisible();
		await expect(triagePanel(page).locator('[data-slot="block-renderer"]').first()).toBeVisible();
		await expect(triagePanel(page).locator('[data-slot="card-envelope"]').first()).toBeVisible();
	});

	test('renders vote blocks for musyawarah flow', async ({ page }) => {
		await authenticateAndOpenTriage(page);
		await completeTriageToReady(page, 'musyawarah', 'Tambahkan poin mufakat');

		await expect(triagePanel(page).getByText('CHAT · Vote Card')).toBeVisible();
		await expect(triagePanel(page).locator('[data-slot="vote-block"]').first()).toBeVisible();
		await expect(triagePanel(page).locator('[data-card-type="vote_card"]').first()).toBeVisible();
	});

	test('renders form blocks for catat data flow', async ({ page }) => {
		await authenticateAndOpenTriage(page);
		await completeTriageToReady(page, 'catat', 'Lengkapi sumber data');

		await expect(triagePanel(page).getByTestId('triage-structured-preview').first()).toBeVisible();
		await expect(triagePanel(page).locator('[data-slot="form-block"]').first()).toBeVisible();
		await expect(triagePanel(page).locator('[data-slot="card-envelope"]').first()).toBeVisible();
	});
});
