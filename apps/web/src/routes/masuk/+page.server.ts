import { fail, redirect } from '@sveltejs/kit';
import { HOME_PATH, SESSION_COOKIE_NAME } from '$lib/auth';
import type { Actions } from './$types';

const getSessionCookieName = () => process.env.GR_SESSION_COOKIE_NAME ?? SESSION_COOKIE_NAME;
const REFRESH_COOKIE_NAME = 'gr_refresh';

export const actions = {
	default: async ({ request, fetch, cookies }) => {
		const form = await request.formData();
		const email = String(form.get('email') ?? '').trim();
		const pass = String(form.get('pass') ?? '');

		if (!email || !pass) {
			return fail(400, { error: 'Email dan password wajib diisi.' });
		}

		const response = await fetch('/v1/auth/signin', {
			method: 'POST',
			headers: {
				accept: 'application/json',
				'content-type': 'application/json'
			},
			body: JSON.stringify({ email, pass })
		});

		if (!response.ok) {
			return fail(response.status, { error: 'Email atau password salah.' });
		}

		const data: {
			access_token?: string;
			refresh_token?: string | null;
		} = await response.json();

		const accessToken = data.access_token;
		if (!accessToken) {
			return fail(500, { error: 'Login gagal: token tidak tersedia.' });
		}

		const cookieOptions = {
			path: '/',
			httpOnly: true,
			sameSite: 'lax' as const,
			secure: !import.meta.env.DEV,
			maxAge: 60 * 60 * 24 * 30
		};

		cookies.set(getSessionCookieName(), accessToken, cookieOptions);
		if (data.refresh_token) {
			cookies.set(REFRESH_COOKIE_NAME, data.refresh_token, cookieOptions);
		}

		throw redirect(303, HOME_PATH);
	}
} satisfies Actions;
