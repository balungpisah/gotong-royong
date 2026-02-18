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

/** Mock session injected automatically in dev mode when no real JWT is present. */
const DEV_MOCK_SESSION: AuthSession = {
	token: 'dev-mock-token',
	user: {
		id: 'dev-user-001',
		role: 'user',
		exp: Math.floor(Date.now() / 1000) + 86400
	}
};

const NO_STORE_CACHE_CONTROL = 'no-store, no-cache, must-revalidate, private, max-age=0';

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
	const session =
		(await resolveAuthSession(event.cookies, event.request.headers)) ??
		(import.meta.env.DEV ? DEV_MOCK_SESSION : null);
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
