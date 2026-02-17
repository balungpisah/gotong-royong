import type { AuthRole, AuthSession, SessionUser } from '$lib/auth';

// See https://svelte.dev/docs/kit/types#app.d.ts
// for information about these interfaces
declare global {
	namespace App {
		// interface Error {}
		interface Locals {
			session: AuthSession | null;
			user: SessionUser | null;
			isAuthenticated: boolean;
			hasRole: (roles: ReadonlyArray<AuthRole>) => boolean;
		}
		// interface PageData {}
		// interface PageState {}
		// interface Platform {}
	}
}

export {};
