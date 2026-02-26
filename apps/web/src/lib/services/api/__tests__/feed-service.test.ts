import { describe, it, expect, vi } from 'vitest';
import type { ApiClient } from '$lib/api';
import { ApiFeedService } from '../feed-service';

const makeApiClient = () => {
	const get = vi.fn();
	const post = vi.fn();
	const client = {
		request: vi.fn(),
		get,
		post,
		put: vi.fn(),
		patch: vi.fn(),
		delete: vi.fn()
	} as unknown as ApiClient;

	return { client, get, post };
};

describe('ApiFeedService', () => {
	it('maps feed payload to FeedItem with enrichment fields', async () => {
		const { client, get } = makeApiClient();
		get.mockResolvedValueOnce({
			items: [
				{
					kind: 'witness',
					stream_id: 'w-feed-1',
					sort_timestamp: '2023-11-14T22:13:20.000Z',
					data: {
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
							dev_meta: {
								is_seed: true,
								seed_batch_id: 'db-seed-2026-02-26',
								seed_origin: 'db'
							},
							program_refs: [
								{
									program_id: 'program:mbg',
									label: 'Makan Bergizi Gratis',
									source: 'llm_inferred',
									confidence: 0.82
								}
							],
							stempel_state: {
								state: 'objection_window',
								min_participants: 3,
								participant_count: 5,
								objection_count: 0,
								objection_deadline_ms: 1_700_000_123_000
							},
							impact_verification: {
								status: 'open',
								opened_at_ms: 1_700_000_000_000,
								closes_at_ms: 1_700_086_400_000,
								yes_count: 2,
								no_count: 0,
								min_vouches: 3
							},
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
				}
			],
			next_cursor: 'next-1',
			has_more: true
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
			kind: 'witness',
			stream_id: 'w-feed-1',
			sort_timestamp: '2023-11-14T22:13:20.000Z'
		});
		if (page.items[0].kind !== 'witness') {
			throw new Error('expected witness stream item');
		}
		expect(page.items[0].data).toMatchObject({
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
			dev_meta: {
				is_seed: true,
				seed_batch_id: 'db-seed-2026-02-26',
				seed_origin: 'db'
			},
			program_refs: [
				{
					program_id: 'program:mbg',
					label: 'Makan Bergizi Gratis',
					source: 'llm_inferred',
					confidence: 0.82
				}
			],
			stempel_state: {
				state: 'objection_window',
				min_participants: 3,
				participant_count: 5,
				objection_count: 0
			},
			impact_verification: {
				status: 'open',
				yes_count: 2,
				no_count: 0,
				min_vouches: 3
			},
			latest_event: {
				event_id: 'feed-1',
				event_type: 'community_note',
				actor_name: 'Rina'
			}
		});
		expect(page.items[0].data.entity_tags).toHaveLength(1);
		expect(page.items[0].data.latest_event.timestamp).toBe('2023-11-14T22:13:20.000Z');
	});

	it('applies fallback mapping when enrichment is absent', async () => {
		const { client, get } = makeApiClient();
		get.mockResolvedValueOnce({
			items: [
				{
					kind: 'witness',
					stream_id: 'w-feed-2',
					sort_timestamp: '2023-11-14T22:13:21.000Z',
					data: {
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
				}
			],
			next_cursor: null
		});

		const service = new ApiFeedService(client);
		const page = await service.list();

		expect(page.cursor).toBeUndefined();
		expect(page.items[0]).toMatchObject({ kind: 'witness' });
		if (page.items[0].kind !== 'witness') {
			throw new Error('expected witness stream item');
		}
		expect(page.items[0].data).toMatchObject({
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

	it('drops invalid dev_meta payload fields safely', async () => {
		const { client, get } = makeApiClient();
		get.mockResolvedValueOnce({
			items: [
				{
					kind: 'witness',
					stream_id: 'w-feed-3',
					sort_timestamp: '2023-11-14T22:13:22.000Z',
					data: {
						feed_id: 'feed-3',
						source_type: 'ontology_note',
						source_id: 'source-3',
						actor_id: 'user-3',
						actor_username: 'Sinta',
						title: 'Catatan',
						privacy_level: 'public',
						occurred_at_ms: 1_700_000_002_000,
						created_at_ms: 1_700_000_002_100,
						payload: {
							dev_meta: {
								is_seed: 'yes',
								seed_origin: 'unknown-origin'
							}
						}
					}
				}
			]
		});

		const service = new ApiFeedService(client);
		const page = await service.list();

		if (page.items[0].kind !== 'witness') {
			throw new Error('expected witness stream item');
		}
		expect(page.items[0].data.dev_meta).toBeUndefined();
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

	it('writes monitor/follow preferences to backend endpoints', async () => {
		const { client, post } = makeApiClient();
		post.mockResolvedValue({});

		const service = new ApiFeedService(client);
		await service.setMonitorPreference('witness-1', true);
		await service.setEntityFollowPreference('ent-rt05', false);

		expect(post).toHaveBeenNthCalledWith(1, '/feed/preferences/monitor/witness-1', {
			body: { monitored: true }
		});
		expect(post).toHaveBeenNthCalledWith(2, '/feed/preferences/follow/ent-rt05', {
			body: { followed: false }
		});
	});
});
