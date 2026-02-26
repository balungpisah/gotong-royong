import { base } from '$app/paths';
import { sequence } from '@sveltejs/kit/hooks';
import { type Handle } from '@sveltejs/kit';
import {
	HOME_PATH,
	LOGIN_PATH,
	hasAnyRole,
	isProtectedPath,
	isPublicOnlyPath,
	requiredRolesForPath,
	type AuthSession
} from '$lib/auth';
import { resolveAuthSession } from '$lib/auth/server';
import { paraglideMiddleware } from '$lib/paraglide/server';

const NO_STORE_CACHE_CONTROL = 'no-store, no-cache, must-revalidate, private, max-age=0';
const DEV_BYPASS_TOKEN = 'dev-bypass-token';
const DEV_BYPASS_USER_ID = 'dev-user';

const envFlagEnabled = (value: string | undefined) => {
	if (!value) {
		return false;
	}

	return ['1', 'true', 'yes', 'on'].includes(value.trim().toLowerCase());
};

const devAuthBypassEnabled = () =>
	process.env.NODE_ENV === 'development' &&
	envFlagEnabled(process.env.GR_AUTH_DEV_BYPASS_ENABLED ?? process.env.AUTH_DEV_BYPASS_ENABLED);

const devBypassSession = (): AuthSession => ({
	token: DEV_BYPASS_TOKEN,
	user: {
		id: process.env.GR_AUTH_DEV_BYPASS_USER_ID?.trim() || DEV_BYPASS_USER_ID,
		role: 'user',
		exp: Math.floor(Date.now() / 1000) + 60 * 60 * 12
	}
});

const stripBasePath = (pathname: string) => {
	if (!base) {
		return pathname;
	}

	if (pathname === base) {
		return '/';
	}

	if (!pathname.startsWith(`${base}/`)) {
		return pathname;
	}

	const unbased = pathname.slice(base.length);
	return unbased || '/';
};

const withBase = (pathname: string) => `${base}${pathname}`;

const applyNoStoreHeaders = (response: Response) => {
	response.headers.set('cache-control', NO_STORE_CACHE_CONTROL);
	response.headers.set('pragma', 'no-cache');
	response.headers.set('expires', '0');
	return response;
};

const noStoreRedirect = (location: string) =>
	applyNoStoreHeaders(
		new Response(null, {
			status: 303,
			headers: { location }
		})
	);

const authHandle: Handle = async ({ event, resolve }) => {
	const resolvedSession = await resolveAuthSession(event.cookies, event.request.headers);
	const session = resolvedSession ?? (devAuthBypassEnabled() ? devBypassSession() : null);
	const user = session?.user ?? null;
	const role = user?.role ?? 'anonymous';

	event.locals.session = session;
	event.locals.user = user;
	event.locals.isAuthenticated = Boolean(session);
	event.locals.hasRole = (roles) => hasAnyRole(role, roles);

	const pathname = stripBasePath(event.url.pathname);
	const requiredRoles = requiredRolesForPath(pathname);
	const isSensitivePath =
		isPublicOnlyPath(pathname) || isProtectedPath(pathname) || Boolean(requiredRoles);

	if (!session && isProtectedPath(pathname)) {
		return noStoreRedirect(withBase(LOGIN_PATH));
	}

	if (session && isPublicOnlyPath(pathname)) {
		return noStoreRedirect(withBase(HOME_PATH));
	}

	if (requiredRoles && !event.locals.hasRole(requiredRoles)) {
		return noStoreRedirect(withBase(HOME_PATH));
	}

	const response = await resolve(event);
	if (isSensitivePath) {
		return applyNoStoreHeaders(response);
	}

	return response;
};

const paraglideHandle: Handle = async ({ event, resolve }) =>
	paraglideMiddleware(event.request, ({ request, locale }) => {
		event.request = request;

		return resolve(event, {
			transformPageChunk: ({ html }) => html.replace('%lang%', locale)
		});
	});

export const handle: Handle = sequence(authHandle, paraglideHandle);
