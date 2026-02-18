import type { TriageService } from '../types';
import type { TriageResult, ContextBarState } from '$lib/types';
import { mockPathPlan } from '$lib/fixtures';

const delay = (ms: number = 200) => new Promise<void>((resolve) => setTimeout(resolve, ms));

const BAR_PROGRESSION: ContextBarState[] = ['listening', 'probing', 'leaning', 'ready'];

export class MockTriageService implements TriageService {
	private currentStep = 0;

	async startTriage(content: string): Promise<TriageResult> {
		await delay(500);
		this.currentStep = 1;
		return {
			bar_state: 'probing',
			route: 'komunitas',
			confidence: { score: 0.4, label: 'Menganalisis...' }
		};
	}

	async updateTriage(sessionId: string, answer: string): Promise<TriageResult> {
		await delay(400);
		this.currentStep = Math.min(this.currentStep + 1, BAR_PROGRESSION.length - 1);
		const barState = BAR_PROGRESSION[this.currentStep];

		if (barState === 'ready') {
			return {
				bar_state: 'ready',
				route: 'komunitas',
				track_hint: 'tuntaskan',
				seed_hint: 'Keresahan',
				confidence: { score: 0.92, label: 'Tuntaskan · 92%' },
				proposed_plan: mockPathPlan
			};
		}

		return {
			bar_state: barState,
			route: 'komunitas',
			track_hint: 'tuntaskan',
			confidence: {
				score: 0.4 + this.currentStep * 0.2,
				label: `Menganalisis... ${Math.round((0.4 + this.currentStep * 0.2) * 100)}%`
			}
		};
	}
}
