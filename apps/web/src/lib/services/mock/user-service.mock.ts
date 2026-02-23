import type { UserService } from '../types';
import type { UserProfile, TandangProfile } from '$lib/types';
import { mockUsers, mockCurrentUser, mockTandangProfiles, mockCurrentTandangProfile } from '$lib/fixtures';

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

	async getTandangProfile(userId: string): Promise<TandangProfile> {
		await delay();
		const profile = mockTandangProfiles.find((p) => p.user_id === userId);
		if (!profile) {
			throw new Error(`Tandang profile not found: ${userId}`);
		}
		return profile;
	}

	async getCurrentTandangProfile(): Promise<TandangProfile> {
		await delay();
		return mockCurrentTandangProfile;
	}
}
