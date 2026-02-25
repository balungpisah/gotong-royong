import { expect, test, type APIRequestContext } from '@playwright/test';

const DEFAULT_SESSION_COOKIE_NAME = 'gr_session';
const LIVE_API_MODE = process.env.PLAYWRIGHT_LIVE_API === 'true';
const SIGNUP_PASSWORD = 'secret123';
const SIGNUP_COMMUNITY_ID = 'rt05';

const createLiveAccessToken = async (request: APIRequestContext) => {
	const runId = `${Date.now()}-${Math.random().toString(36).slice(2, 10)}`;
	const response = await request.post('/v1/auth/signup', {
		headers: {
			accept: 'application/json',
			'content-type': 'application/json'
		},
		data: {
			email: `e2e-live-${runId}@example.com`,
			pass: SIGNUP_PASSWORD,
			username: `e2e_live_${runId}`,
			community_id: SIGNUP_COMMUNITY_ID
		}
	});

	const rawBody = await response.text();
	if (!response.ok) {
		throw new Error(
			`Live auth bootstrap failed: POST /v1/auth/signup returned ${response.status()} with body ${rawBody}`
		);
	}

	let parsedBody: { access_token?: string };
	try {
		parsedBody = JSON.parse(rawBody) as { access_token?: string };
	} catch {
		throw new Error(
			`Live auth bootstrap failed: POST /v1/auth/signup returned non-JSON body ${rawBody}`
		);
	}

	if (!parsedBody.access_token) {
		throw new Error(
			'Live auth bootstrap failed: POST /v1/auth/signup response missing access_token'
		);
	}

	return parsedBody.access_token;
};

test.describe('live API proxy smoke', () => {
	test.skip(!LIVE_API_MODE, 'Set PLAYWRIGHT_LIVE_API=true to run live API proxy smoke.');

	test('home shell issues real /v1 hot-path requests through Vite proxy', async ({
		context,
		page,
		request
	}) => {
		const baseURL = test.info().project.use.baseURL;
		if (!baseURL) {
			throw new Error('Playwright baseURL is required for cookie injection.');
		}

		const token = await createLiveAccessToken(request);
		await context.addCookies([
			{
				name: process.env.GR_SESSION_COOKIE_NAME ?? DEFAULT_SESSION_COOKIE_NAME,
				value: token,
				url: `${baseURL}/`
			}
		]);

		await page.route('**/v1/**', async (route) => {
			const headers = {
				...route.request().headers(),
				authorization: `Bearer ${token}`
			};
			await route.continue({ headers });
		});

		const expectedPaths = ['/v1/auth/me', '/v1/feed', '/v1/notifications'];
		const hits = new Map<string, number>();
		const statuses = new Map<string, Set<number>>();

		page.on('response', (response) => {
			const url = new URL(response.url());
			for (const path of expectedPaths) {
				if (url.pathname === path) {
					const authorization = response.request().headers()['authorization'];
					if (authorization !== `Bearer ${token}`) {
						continue;
					}
					hits.set(path, (hits.get(path) ?? 0) + 1);
					const seenStatuses = statuses.get(path) ?? new Set<number>();
					seenStatuses.add(response.status());
					statuses.set(path, seenStatuses);
				}
			}
		});

		await page.goto('/');
		await expect(page).toHaveURL(/\/$/);
		await expect(page.getByRole('link', { name: 'Gotong Royong' })).toBeVisible({
			timeout: 20_000
		});

		for (const path of expectedPaths) {
			await expect
				.poll(() => hits.get(path) ?? 0, {
					timeout: 20_000,
					message: `Expected frontend to request ${path}`
				})
				.toBeGreaterThan(0);
			await expect
				.poll(() => (statuses.get(path)?.has(200) ? 200 : undefined), {
					timeout: 20_000,
					message: `Expected ${path} to return 200 via proxy`
				})
				.toBe(200);
			const observedStatuses = [...(statuses.get(path) ?? new Set<number>())];
			expect(
				observedStatuses.every((status) => status === 200),
				`Expected ${path} to return only 200 statuses, got [${observedStatuses.join(', ')}]`
			).toBe(true);
		}
	});
});
