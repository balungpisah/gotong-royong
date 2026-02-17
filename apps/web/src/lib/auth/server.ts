import { jwtVerify, type JWTPayload } from 'jose';
import type { Cookies } from '@sveltejs/kit';
import { type AuthSession, type SessionUser, isAuthRole, SESSION_COOKIE_NAME } from '$lib/auth';

const JWT_ALGORITHM = 'HS256';
const textEncoder = new TextEncoder();

interface JwtClaims extends JWTPayload {
	sub?: string;
	role?: string;
	exp?: number;
}

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

const toSessionUser = (payload: JwtClaims): SessionUser | null => {
	if (!payload.sub || !payload.role || !payload.exp) {
		return null;
	}

	if (!isAuthRole(payload.role) || payload.role === 'anonymous') {
		return null;
	}

	if (!Number.isFinite(payload.exp)) {
		return null;
	}

	return {
		id: payload.sub,
		role: payload.role,
		exp: payload.exp
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

	const jwtSecret = process.env.JWT_SECRET;
	if (!jwtSecret) {
		return null;
	}

	try {
		const { payload } = await jwtVerify<JwtClaims>(token, textEncoder.encode(jwtSecret), {
			algorithms: [JWT_ALGORITHM]
		});

		const user = toSessionUser(payload);
		if (!user) {
			return null;
		}

		return {
			token,
			user
		};
	} catch {
		return null;
	}
};
