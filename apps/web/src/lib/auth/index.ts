export const AUTH_ROLES = ['anonymous', 'user', 'moderator', 'admin', 'system'] as const;

export type AuthRole = (typeof AUTH_ROLES)[number];

export interface SessionUser {
	id: string;
	role: Exclude<AuthRole, 'anonymous'>;
	exp: number;
}

export interface AuthSession {
	token: string;
	user: SessionUser;
}

export const SESSION_COOKIE_NAME = 'gr_session';

export const LOGIN_PATH = '/masuk';
export const HOME_PATH = '/';

const PROTECTED_PATH_PREFIXES = ['/t', '/notifikasi', '/profil'] as const;

const ROLE_GUARD_RULES: ReadonlyArray<{
	prefix: string;
	roles: ReadonlyArray<AuthRole>;
}> = [{ prefix: '/admin', roles: ['admin', 'system'] }];

const matchesPathPrefix = (pathname: string, prefix: string) =>
	pathname === prefix || pathname.startsWith(`${prefix}/`);

export const isAuthRole = (value: unknown): value is AuthRole =>
	typeof value === 'string' && AUTH_ROLES.includes(value as AuthRole);

export const hasAnyRole = (role: AuthRole | null | undefined, roles: ReadonlyArray<AuthRole>) =>
	Boolean(role && roles.includes(role));

export const isProtectedPath = (pathname: string) =>
	pathname === '/' || PROTECTED_PATH_PREFIXES.some((prefix) => matchesPathPrefix(pathname, prefix));

export const requiredRolesForPath = (pathname: string) =>
	ROLE_GUARD_RULES.find((rule) => matchesPathPrefix(pathname, rule.prefix))?.roles ?? null;

export const isPublicOnlyPath = (pathname: string) => matchesPathPrefix(pathname, LOGIN_PATH);
