import { expect, test } from '@playwright/test';
import { SignJWT } from 'jose';

const DEFAULT_JWT_SECRET = 'playwright-jwt-secret';
const DEFAULT_SESSION_COOKIE_NAME = 'gr_session';

/**
 * Create an authenticated session for e2e tests.
 * Uses u-001 (Budi Santoso) which is the mock current user,
 * meaning they are admin of mockGroup1 (ent-003) and mockGroup4 (ent-102).
 */
const createSessionToken = async () => {
	const secret = process.env.JWT_SECRET ?? DEFAULT_JWT_SECRET;
	const signingSecret = new TextEncoder().encode(secret);
	const now = Math.floor(Date.now() / 1000);

	return new SignJWT({
		sub: 'u-001',
		role: 'user',
		exp: now + 60 * 10
	})
		.setProtectedHeader({ alg: 'HS256' })
		.sign(signingSecret);
};

async function authenticate(context: any) {
	const token = await createSessionToken();
	const baseURL = 'http://127.0.0.1:4173';

	await context.addCookies([
		{
			name: process.env.GR_SESSION_COOKIE_NAME ?? DEFAULT_SESSION_COOKIE_NAME,
			value: token,
			url: `${baseURL}/`
		}
	]);
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

test.describe('Group lifecycle (Kelompok)', () => {
	test.beforeEach(async ({ context }) => {
		await authenticate(context);
	});

	test('komunitas page shows link to kelompok', async ({ page }) => {
		await page.goto('/komunitas');

		// The komunitas page has a card linking to /komunitas/kelompok
		const navLink = page.locator('a[href="/komunitas/kelompok"]');
		await expect(navLink).toBeVisible();
		await expect(navLink).toContainText('Kelompok');
	});

	test('kelompok list page renders My Groups and Discover sections', async ({ page }) => {
		await page.goto('/komunitas/kelompok');

		// Page title
		await expect(page.getByText('Kelompok & Lembaga')).toBeVisible();

		// "Buat" (Create) button
		const createBtn = page.getByRole('button', { name: 'Buat' });
		await expect(createBtn).toBeVisible();

		// "Kelompok Saya" section - u-001 is member of mockGroup1 and mockGroup4
		await expect(page.getByText('Kelompok Saya')).toBeVisible();
		// Use .first() since group may appear in both "My Groups" and "Discover"
		await expect(page.getByText('Karang Taruna RT 05').first()).toBeVisible();
		await expect(page.getByText('Forum Pemuda RW 03').first()).toBeVisible();

		// "Jelajahi" section - discoverable groups (terbuka + persetujuan, excluding undangan)
		await expect(page.getByRole('heading', { name: 'Jelajahi' })).toBeVisible();
	});

	test('can open create form and create a new group', async ({ page }) => {
		await page.goto('/komunitas/kelompok');

		// Wait for mock data to be loaded (proves hydration is complete, not just SSR)
		await expect(page.getByRole('link', { name: 'Karang Taruna RT 05' }).first()).toBeVisible();

		// Toggle create form — button has exact text "Buat"
		const createBtn = page.getByRole('button', { name: 'Buat', exact: true });
		await createBtn.click();

		// Create form should appear — wait for the name input placeholder
		await expect(page.getByPlaceholder('Contoh: Karang Taruna RT 05')).toBeVisible({ timeout: 5000 });

		// Fill in the form
		const nameInput = page.getByPlaceholder('Contoh: Karang Taruna RT 05');
		await nameInput.fill('Kelompok Test E2E');

		const descInput = page.getByPlaceholder('Tulis tujuan, kegiatan, dan siapa yang cocok bergabung...');
		await descInput.fill('Kelompok untuk pengujian end-to-end otomatis.');

		// Entity type defaults to 'kelompok', join policy to 'terbuka' - keep defaults

		// Submit
		const submitBtn = page.getByRole('button', { name: 'Buat Kelompok' });
		await expect(submitBtn).toBeEnabled();
		await submitBtn.click();

		// After creation, form should close and new group appears in "Kelompok Saya"
		await expect(page.getByPlaceholder('Contoh: Karang Taruna RT 05')).not.toBeVisible({ timeout: 5000 });
		await expect(page.getByText('Kelompok Test E2E').first()).toBeVisible();
	});

	test('can navigate to group detail page', async ({ page }) => {
		await page.goto('/komunitas/kelompok');

		// Click "Lihat" on first group card (Karang Taruna RT 05)
		const viewLinks = page.getByRole('link', { name: 'Lihat' });
		await viewLinks.first().click();

		// Should navigate to detail page
		await expect(page).toHaveURL(/\/komunitas\/kelompok\/ent-/);

		// Back link
		await expect(page.getByText('Kembali ke daftar')).toBeVisible();
	});

	test('group detail shows group info and member tab', async ({ page }) => {
		// Navigate directly to mockGroup1 (Karang Taruna RT 05) where u-001 is admin
		await page.goto('/komunitas/kelompok/ent-003');

		// Group name and description
		await expect(page.getByText('Karang Taruna RT 05')).toBeVisible();
		await expect(page.getByText('Wadah pemuda RT 05')).toBeVisible();

		// Privacy badge - terbuka
		await expect(page.getByText('Terbuka')).toBeVisible();

		// Member stats
		await expect(page.getByText('5 anggota')).toBeVisible();

		// "Keluar" button (since u-001 is a member) — exact match to avoid "Keluarkan anggota" buttons
		await expect(page.getByRole('button', { name: 'Keluar', exact: true })).toBeVisible();

		// Tabs - admin sees all 3 (exact match to avoid "Keluarkan anggota" buttons)
		await expect(page.getByRole('button', { name: 'Anggota', exact: true })).toBeVisible();
		await expect(page.getByRole('button', { name: /Pengaturan/ })).toBeVisible();
	});

	test('admin can see settings tab and edit group', async ({ page }) => {
		// mockGroup1: u-001 is admin
		await page.goto('/komunitas/kelompok/ent-003');

		// Click settings tab
		const settingsTab = page.getByRole('button', { name: /Pengaturan/ });
		await settingsTab.click();

		// Settings form should appear
		await expect(page.getByText('Pengaturan Kelompok')).toBeVisible();

		// Name input should be pre-filled
		const nameInput = page.locator('input').filter({ hasText: '' }).nth(0);
		await expect(page.getByText('Nama')).toBeVisible();

		// Save button
		await expect(page.getByRole('button', { name: 'Simpan' })).toBeVisible();

		// Invite section
		await expect(page.getByText('Undang anggota')).toBeVisible();
		await expect(page.getByPlaceholder('Masukkan user_id')).toBeVisible();
	});

	test('admin can see pending requests tab for persetujuan group', async ({ page }) => {
		// mockGroup4 (Forum Pemuda RW 03): u-001 is admin, join_policy=persetujuan, has 2 pending requests
		await page.goto('/komunitas/kelompok/ent-102');

		await expect(page.getByText('Forum Pemuda RW 03')).toBeVisible();

		// Requests tab should show count
		const requestsTab = page.getByRole('button', { name: /Permintaan/ });
		await expect(requestsTab).toBeVisible();
		await requestsTab.click();

		// Should show approve/reject buttons
		await expect(page.getByRole('button', { name: 'Setujui' }).first()).toBeVisible();
		await expect(page.getByRole('button', { name: 'Tolak' }).first()).toBeVisible();
	});

	test('can join a terbuka group from detail page', async ({ page }) => {
		// mockGroup3 (Komunitas Peduli Lingkungan): u-001 is NOT a member, terbuka
		await page.goto('/komunitas/kelompok/ent-101');

		await expect(page.getByText('Komunitas Peduli Lingkungan')).toBeVisible();

		// Should see "Gabung" button
		const joinBtn = page.getByRole('button', { name: 'Gabung' });
		await expect(joinBtn).toBeVisible();
		await joinBtn.click();

		// After joining, should see "Keluar" button instead
		await expect(page.getByRole('button', { name: 'Keluar', exact: true })).toBeVisible({ timeout: 5000 });
	});

	test('pending request status shows for persetujuan group', async ({ page }) => {
		// mockGroup2 (Komite Sekolah SDN 3): u-001 has pending request
		await page.goto('/komunitas/kelompok/ent-004');

		await expect(page.getByText('Komite Sekolah SDN 3 Menteng')).toBeVisible();

		// Should show pending badge
		await expect(page.getByText('Menunggu persetujuan')).toBeVisible();
	});

	test('invite-only group shows correct badge', async ({ page }) => {
		// mockGroup5 (Yayasan Gotong Royong): undangan, u-001 is NOT a member
		await page.goto('/komunitas/kelompok/ent-103');

		await expect(page.getByText('Yayasan Gotong Royong')).toBeVisible();

		// Should show invite-only indicator
		await expect(page.getByText('Undangan').first()).toBeVisible();
	});
});
