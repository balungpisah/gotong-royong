import { describe, it, expect, vi } from 'vitest';
import type { ApiClient } from '$lib/api';
import { ApiFeedService } from '../feed-service';

const makeApiClient = () => {
	const get = vi.fn();
	const client = {
		request: vi.fn(),
		get,
		post: vi.fn(),
		put: vi.fn(),
		patch: vi.fn(),
		delete: vi.fn()
	} as unknown as ApiClient;

	return { client, get };
};

describe('ApiFeedService', () => {
	it('maps feed payload to FeedItem with enrichment fields', async () => {
		const { client, get } = makeApiClient();
		get.mockResolvedValueOnce({
			items: [
				{
					feed_id: 'feed-1',
					source_type: 'ontology_note',
					source_id: 'note-1',
					actor_id: 'user-1',
					actor_username: 'Rina',
					title: 'Judul dari backend',
					summary: 'Ringkasan backend',
					privacy_level: 'public',
					occurred_at_ms: 1_700_000_000_000,
					created_at_ms: 1_700_000_000_100,
					participant_ids: ['user-2'],
					payload: {
						witness_id: 'witness-1',
						monitored: true,
						enrichment: {
							title: 'Judul enrichment',
							trajectory_type: 'data',
							icon: 'scale',
							hook_line: 'Hook kuat',
							body: 'Body enrichment',
							sentiment: 'curious',
							intensity: 3,
							entity_tags: [{ label: 'RT 05', entity_type: 'lingkungan' }]
						}
					}
				}
			],
			next_cursor: 'next-1'
		});

		const service = new ApiFeedService(client);
		const page = await service.list({ cursor: 'cursor-0', limit: 10 });

		expect(get).toHaveBeenCalledWith('/feed', {
			query: {
				cursor: 'cursor-0',
				limit: 10
			}
		});
		expect(page.total).toBe(1);
		expect(page.cursor).toBe('next-1');
		expect(page.items[0]).toMatchObject({
			witness_id: 'witness-1',
			title: 'Judul enrichment',
			trajectory_type: 'data',
			icon: 'scale',
			rahasia_level: 'L0',
			source: 'sekitar',
			hook_line: 'Hook kuat',
			body: 'Body enrichment',
			sentiment: 'curious',
			intensity: 3,
			monitored: true,
			latest_event: {
				event_id: 'feed-1',
				event_type: 'community_note',
				actor_name: 'Rina'
			}
		});
		expect(page.items[0].entity_tags).toHaveLength(1);
		expect(page.items[0].latest_event.timestamp).toBe('2023-11-14T22:13:20.000Z');
	});

	it('applies fallback mapping when enrichment is absent', async () => {
		const { client, get } = makeApiClient();
		get.mockResolvedValueOnce({
			items: [
				{
					feed_id: 'feed-2',
					source_type: 'vouch',
					source_id: 'source-2',
					actor_id: 'user-9',
					actor_username: 'Budi',
					title: 'Ada vouch',
					privacy_level: 'private',
					occurred_at_ms: 1_700_000_001_000,
					created_at_ms: 1_700_000_001_100,
					participant_ids: [],
					payload: {}
				}
			],
			next_cursor: null
		});

		const service = new ApiFeedService(client);
		const page = await service.list();

		expect(page.cursor).toBeUndefined();
		expect(page.items[0]).toMatchObject({
			witness_id: 'source-2',
			title: 'Ada vouch',
			trajectory_type: 'advokasi',
			rahasia_level: 'L1',
			source: 'terlibat',
			urgency: 'ramai',
			latest_event: {
				event_type: 'joined',
				verb: 'memberi vouch'
			}
		});
	});

	it('maps suggestions from backend endpoint', async () => {
		const { client, get } = makeApiClient();
		get.mockResolvedValueOnce([
			{
				entity_id: 'ent-rt05',
				entity_type: 'lingkungan',
				label: 'RT 05 Menteng',
				followed: false,
				description: 'Komunitas warga RT 05',
				witness_count: 12,
				follower_count: 44
			},
			{
				entity_type: 'topik',
				label: 'Infrastruktur',
				witness_count: 3
			}
		]);

		const service = new ApiFeedService(client);
		const suggestions = await service.listSuggestions();

		expect(get).toHaveBeenCalledWith('/feed/suggestions');
		expect(suggestions).toHaveLength(2);
		expect(suggestions[0]).toMatchObject({
			entity_id: 'ent-rt05',
			entity_type: 'lingkungan',
			label: 'RT 05 Menteng',
			followed: false,
			witness_count: 12,
			follower_count: 44
		});
		expect(suggestions[1]).toMatchObject({
			entity_id: 'topik:infrastruktur',
			entity_type: 'topik',
			label: 'Infrastruktur',
			followed: false,
			witness_count: 3,
			follower_count: 3
		});
	});
});
