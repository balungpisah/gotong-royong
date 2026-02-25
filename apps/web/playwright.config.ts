import { defineConfig, devices } from '@playwright/test';

const PORT = 4173;
const HOST = '127.0.0.1';
const baseURL = `http://${HOST}:${PORT}`;
const TEST_JWT_SECRET = process.env.JWT_SECRET ?? 'playwright-jwt-secret';
const LIVE_API_MODE = process.env.PLAYWRIGHT_LIVE_API === 'true';

if (LIVE_API_MODE && !process.env.GR_API_PROXY_TARGET) {
	throw new Error(
		'PLAYWRIGHT_LIVE_API=true requires GR_API_PROXY_TARGET to point at gotong-api.'
	);
}

export default defineConfig({
	testDir: './tests/e2e',
	fullyParallel: true,
	forbidOnly: Boolean(process.env.CI),
	retries: process.env.CI ? 2 : 0,
	workers: process.env.CI || LIVE_API_MODE ? 1 : undefined,
	reporter: process.env.CI ? [['github'], ['html', { open: 'never' }]] : 'list',
	use: {
		baseURL,
		trace: 'retain-on-failure'
	},
	webServer: {
		command: `bun run dev -- --host ${HOST} --port ${PORT}`,
		url: `${baseURL}/masuk`,
		env: {
			...process.env,
			JWT_SECRET: TEST_JWT_SECRET,
			PUBLIC_GR_USE_API_NOTIFICATIONS: LIVE_API_MODE ? 'true' : 'false',
			PUBLIC_GR_USE_API_FEED: LIVE_API_MODE ? 'true' : 'false',
			PUBLIC_GR_USE_API_CHAT: LIVE_API_MODE ? 'true' : 'false',
			PUBLIC_GR_USE_API_USER: 'true',
			PUBLIC_GR_USE_API_TRIAGE: LIVE_API_MODE ? 'true' : 'false',
			PUBLIC_GR_USE_API_SIGNAL: LIVE_API_MODE ? 'true' : 'false',
			PUBLIC_GR_USE_API_GROUP: LIVE_API_MODE ? 'true' : 'false'
		},
		reuseExistingServer: !process.env.CI,
		timeout: 120000
	},
	projects: [
		{
			name: 'chromium',
			use: { ...devices['Desktop Chrome'] }
		}
	]
});
