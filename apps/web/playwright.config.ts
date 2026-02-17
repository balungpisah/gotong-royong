import { defineConfig, devices } from '@playwright/test';

const PORT = 4173;
const HOST = '127.0.0.1';
const baseURL = `http://${HOST}:${PORT}`;
const TEST_JWT_SECRET = process.env.JWT_SECRET ?? 'playwright-jwt-secret';

export default defineConfig({
	testDir: './tests/e2e',
	fullyParallel: true,
	forbidOnly: Boolean(process.env.CI),
	retries: process.env.CI ? 2 : 0,
	workers: process.env.CI ? 1 : undefined,
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
			JWT_SECRET: TEST_JWT_SECRET
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
