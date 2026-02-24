import type { Cookies } from '@sveltejs/kit';
import { type AuthSession, type SessionUser, isAuthRole, SESSION_COOKIE_NAME } from '$lib/auth';

const getSessionCookieName = () => process.env.GR_SESSION_COOKIE_NAME ?? SESSION_COOKIE_NAME;

const parseBearerToken = (authorizationHeader: string | null) => {
	if (!authorizationHeader) {
		return null;
	}

	const [scheme, token] = authorizationHeader.split(' ');
	if (!scheme || !token || scheme.toLowerCase() !== 'bearer') {
		return null;
	}

	return token.trim();
};

const getRequestToken = (cookies: Cookies, headers: Headers) =>
	cookies.get(getSessionCookieName()) ?? parseBearerToken(headers.get('authorization'));

const decodeBase64UrlJson = (segment: string): unknown => {
	const normalized = segment.replace(/-/g, '+').replace(/_/g, '/');
	const padded = normalized.padEnd(normalized.length + ((4 - (normalized.length % 4)) % 4), '=');
	const json = Buffer.from(padded, 'base64').toString('utf8');
	return JSON.parse(json);
};

type JwtClaims = {
	sub?: string;
	role?: string;
	exp?: number;
	// SurrealDB record auth tokens commonly include these uppercase claims:
	ID?: string;
};

const parseJwtPayload = (token: string): JwtClaims | null => {
	const [, payload] = token.split('.');
	if (!payload) {
		return null;
	}

	try {
		const parsed = decodeBase64UrlJson(payload);
		if (!parsed || typeof parsed !== 'object') {
			return null;
		}
		return parsed as JwtClaims;
	} catch {
		return null;
	}
};

const toSessionUser = (payload: JwtClaims): SessionUser | null => {
	const id = payload.sub ?? payload.ID;
	const exp = payload.exp;
	const role = payload.role;

	if (!id || typeof id !== 'string') {
		return null;
	}
	if (!exp || !Number.isFinite(exp)) {
		return null;
	}

	// Role is an application concern; SurrealDB-issued tokens might not include it.
	const resolvedRole = role && isAuthRole(role) && role !== 'anonymous' ? role : 'user';

	const normalizedId = id.includes(':') ? id.split(':', 2)[1] : id;

	return {
		id: normalizedId,
		role: resolvedRole,
		exp
	};
};

export const resolveAuthSession = async (
	cookies: Cookies,
	headers: Headers
): Promise<AuthSession | null> => {
	const token = getRequestToken(cookies, headers);
	if (!token) {
		return null;
	}

	const payload = parseJwtPayload(token);
	if (!payload) {
		return null;
	}

	const user = toSessionUser(payload);
	if (!user) {
		return null;
	}

	return {
		token,
		user
	};
};
