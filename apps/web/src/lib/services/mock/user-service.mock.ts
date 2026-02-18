import type { UserService } from '../types';
import type { UserProfile } from '$lib/types';
import { mockUsers, mockCurrentUser } from '$lib/fixtures';

const delay = (ms: number = 200) => new Promise<void>((resolve) => setTimeout(resolve, ms));

export class MockUserService implements UserService {
	async getProfile(userId: string): Promise<UserProfile> {
		await delay();
		const user = mockUsers.find((u) => u.user_id === userId);
		if (!user) {
			throw new Error(`User not found: ${userId}`);
		}
		return user;
	}

	async getCurrentUser(): Promise<UserProfile> {
		await delay();
		return mockCurrentUser;
	}
}
