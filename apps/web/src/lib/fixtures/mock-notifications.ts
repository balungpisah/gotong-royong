/**
 * Mock notification fixtures for the dev gallery.
 * 10 notifications covering all 7 NotificationType variants.
 * Mix: 6 unread, 4 read. All with realistic Indonesian content.
 */

import type { AppNotification } from '$lib/types';

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

const now = Date.now();

/** Returns an ISO timestamp N minutes in the past. */
const ts = (minutesAgo: number): string => new Date(now - minutesAgo * 60 * 1000).toISOString();

/** Returns an ISO timestamp N hours in the past. */
const tsHour = (hoursAgo: number): string => ts(hoursAgo * 60);

// ---------------------------------------------------------------------------
// Individual notifications (exported for component-level testing)
// ---------------------------------------------------------------------------

export const mockNotification1: AppNotification = {
	notification_id: 'notif-001',
	type: 'phase_change',
	title: 'Fase selesai',
	body: "Fase 'Pengumpulan Bukti' selesai pada saksi Jalan Rusak Jl. Mawar RT 05.",
	witness_id: 'witness-001',
	read: false,
	created_at: ts(5)
};

export const mockNotification2: AppNotification = {
	notification_id: 'notif-002',
	type: 'vote_open',
	title: 'Voting baru dibuka',
	body: 'Voting: Lanjutkan penggalangan dana? Berikan suara Anda sebelum 48 jam ke depan.',
	witness_id: 'witness-001',
	read: false,
	created_at: ts(12)
};

export const mockNotification3: AppNotification = {
	notification_id: 'notif-003',
	type: 'evidence_needed',
	title: 'Bukti diperlukan',
	body: 'Bukti tambahan diperlukan untuk checkpoint "Persetujuan musyawarah warga" agar dapat dilanjutkan.',
	witness_id: 'witness-001',
	read: false,
	created_at: ts(30)
};

export const mockNotification4: AppNotification = {
	notification_id: 'notif-004',
	type: 'diff_proposed',
	title: 'Perubahan rencana diusulkan',
	body: 'AI menyarankan perubahan pada rencana: ditambah 2 langkah baru di fase Penggalangan Dana.',
	witness_id: 'witness-001',
	read: false,
	created_at: ts(45)
};

export const mockNotification5: AppNotification = {
	notification_id: 'notif-005',
	type: 'mention',
	title: 'Anda disebut',
	body: 'Ahmad Hidayat menyebut Anda dalam diskusi saksi Pembangunan Taman Warga RW 03.',
	witness_id: 'witness-002',
	read: false,
	created_at: tsHour(2)
};

export const mockNotification6: AppNotification = {
	notification_id: 'notif-006',
	type: 'role_assigned',
	title: 'Peran baru ditetapkan',
	body: 'Anda ditunjuk sebagai koordinator pada saksi Jalan Rusak Jl. Mawar RT 05 oleh Rina Kartika.',
	witness_id: 'witness-001',
	read: true,
	created_at: tsHour(5)
};

export const mockNotification7: AppNotification = {
	notification_id: 'notif-007',
	type: 'system',
	title: 'Selamat datang!',
	body: 'Selamat datang di Gotong Royong! Mulai berkontribusi dengan melaporkan isu di lingkungan Anda.',
	read: true,
	created_at: tsHour(24)
};

export const mockNotification8: AppNotification = {
	notification_id: 'notif-008',
	type: 'phase_change',
	title: 'Fase baru dimulai',
	body: "Fase 'Penggalangan Dana' telah dimulai pada saksi Jalan Rusak Jl. Mawar RT 05.",
	witness_id: 'witness-001',
	read: false,
	created_at: tsHour(3)
};

export const mockNotification9: AppNotification = {
	notification_id: 'notif-009',
	type: 'vote_open',
	title: 'Hasil voting tersedia',
	body: 'Hasil voting "Lanjutkan dengan jalur eskalasi?": Setuju 78% (18/23 suara). Quorum tercapai.',
	witness_id: 'witness-001',
	read: true,
	created_at: tsHour(8)
};

export const mockNotification10: AppNotification = {
	notification_id: 'notif-010',
	type: 'system',
	title: 'Pembaruan aplikasi',
	body: 'Pembaruan aplikasi v2.1 tersedia. Fitur baru: tampilan galeri bukti dan filter notifikasi.',
	read: true,
	created_at: tsHour(48)
};

// ---------------------------------------------------------------------------
// All notifications array (newest first)
// ---------------------------------------------------------------------------

export const mockNotifications: AppNotification[] = [
	mockNotification1,
	mockNotification2,
	mockNotification3,
	mockNotification4,
	mockNotification5,
	mockNotification6,
	mockNotification7,
	mockNotification8,
	mockNotification9,
	mockNotification10
];

// ---------------------------------------------------------------------------
// Filtered: unread only
// ---------------------------------------------------------------------------

export const mockUnreadNotifications: AppNotification[] = mockNotifications.filter((n) => !n.read);
