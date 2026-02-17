import { base } from '$app/paths';
import { sequence } from '@sveltejs/kit/hooks';
import { redirect, type Handle } from '@sveltejs/kit';
import {
	HOME_PATH,
	LOGIN_PATH,
	hasAnyRole,
	isProtectedPath,
	isPublicOnlyPath,
	requiredRolesForPath
} from '$lib/auth';
import { resolveAuthSession } from '$lib/auth/server';
import { paraglideMiddleware } from '$lib/paraglide/server';

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

const authHandle: Handle = async ({ event, resolve }) => {
	const session = await resolveAuthSession(event.cookies, event.request.headers);
	const user = session?.user ?? null;
	const role = user?.role ?? 'anonymous';

	event.locals.session = session;
	event.locals.user = user;
	event.locals.isAuthenticated = Boolean(session);
	event.locals.hasRole = (roles) => hasAnyRole(role, roles);

	const pathname = stripBasePath(event.url.pathname);

	if (!session && isProtectedPath(pathname)) {
		throw redirect(303, withBase(LOGIN_PATH));
	}

	if (session && isPublicOnlyPath(pathname)) {
		throw redirect(303, withBase(HOME_PATH));
	}

	const requiredRoles = requiredRolesForPath(pathname);
	if (requiredRoles && !event.locals.hasRole(requiredRoles)) {
		throw redirect(303, withBase(HOME_PATH));
	}

	return resolve(event);
};

const paraglideHandle: Handle = async ({ event, resolve }) =>
	paraglideMiddleware(event.request, ({ request, locale }) => {
		event.request = request;

		return resolve(event, {
			transformPageChunk: ({ html }) => html.replace('%lang%', locale)
		});
	});

export const handle: Handle = sequence(authHandle, paraglideHandle);
