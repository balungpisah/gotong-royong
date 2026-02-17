import { describe, expect, it } from 'vitest';
import {
	AUTH_ROLES,
	HOME_PATH,
	LOGIN_PATH,
	SESSION_COOKIE_NAME,
	hasAnyRole,
	isAuthRole,
	isProtectedPath,
	isPublicOnlyPath,
	requiredRolesForPath
} from './index';

describe('auth constants', () => {
	it('uses stable baseline paths and cookie name', () => {
		expect(LOGIN_PATH).toBe('/masuk');
		expect(HOME_PATH).toBe('/beranda');
		expect(SESSION_COOKIE_NAME).toBe('gr_session');
		expect(AUTH_ROLES).toContain('admin');
	});
});

describe('auth route helpers', () => {
	it('detects protected routes including nested data requests', () => {
		expect(isProtectedPath('/beranda')).toBe(true);
		expect(isProtectedPath('/beranda/__data.json')).toBe(true);
		expect(isProtectedPath('/masuk')).toBe(false);
	});

	it('detects public-only login routes', () => {
		expect(isPublicOnlyPath('/masuk')).toBe(true);
		expect(isPublicOnlyPath('/masuk/__data.json')).toBe(true);
		expect(isPublicOnlyPath('/beranda')).toBe(false);
	});

	it('returns required roles for guarded routes', () => {
		expect(requiredRolesForPath('/admin')).toEqual(['admin', 'system']);
		expect(requiredRolesForPath('/admin/users')).toEqual(['admin', 'system']);
		expect(requiredRolesForPath('/beranda')).toBeNull();
	});
});

describe('role helpers', () => {
	it('validates supported roles', () => {
		expect(isAuthRole('user')).toBe(true);
		expect(isAuthRole('system')).toBe(true);
		expect(isAuthRole('guest')).toBe(false);
	});

	it('matches membership checks', () => {
		expect(hasAnyRole('admin', ['admin', 'system'])).toBe(true);
		expect(hasAnyRole('user', ['admin', 'system'])).toBe(false);
		expect(hasAnyRole(null, ['admin', 'system'])).toBe(false);
	});
});
