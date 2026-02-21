/**
 * Mock path plan fixtures for the dev gallery.
 * A realistic PathPlan with 2 branches, 3 phases, and 9 checkpoints.
 */

import type { PathPlan, PathPlanEnvelope } from '$lib/types';

export const mockPathPlan: PathPlan = {
	plan_id: 'plan-001',
	version: 3,
	title: 'Rencana Perbaikan Jalan Jl. Mawar',
	summary:
		'Rencana penyelesaian masalah jalan rusak melalui koordinasi warga, penggalangan dana, dan pelaksanaan perbaikan.',
	track_hint: 'tuntaskan',
	seed_hint: 'Keresahan',
	branches: [
		{
			branch_id: 'branch-main',
			label: 'Jalur Utama',
			parent_checkpoint_id: null,
			phases: [
				{
					phase_id: 'phase-1',
					title: 'Pengumpulan Bukti',
					objective: 'Mengumpulkan dokumentasi dan kesaksian warga terdampak',
					status: 'completed',
					source: 'ai',
					locked_fields: [],
					checkpoints: [
						{
							checkpoint_id: 'cp-1',
							title: 'Foto dan video kondisi jalan',
							status: 'completed',
							source: 'human',
							locked_fields: []
						},
						{
							checkpoint_id: 'cp-2',
							title: 'Minimal 5 kesaksian warga',
							status: 'completed',
							source: 'ai',
							locked_fields: [],
							evidence_required: true
						},
						{
							checkpoint_id: 'cp-3',
							title: 'Surat pengantar RT',
							status: 'completed',
							source: 'human',
							locked_fields: ['title']
						}
					]
				},
				{
					phase_id: 'phase-2',
					title: 'Penggalangan Dana',
					objective: 'Mengumpulkan dana swadaya dan mengajukan bantuan kelurahan',
					status: 'active',
					source: 'ai',
					locked_fields: [],
					checkpoints: [
						{
							checkpoint_id: 'cp-4',
							title: 'Target dana Rp 15.000.000',
							status: 'open',
							source: 'ai',
							locked_fields: [],
							description: 'Target minimal dari swadaya warga'
						},
						{
							checkpoint_id: 'cp-5',
							title: 'Proposal ke kelurahan',
							status: 'open',
							source: 'ai',
							locked_fields: []
						},
						{
							checkpoint_id: 'cp-6',
							title: 'Persetujuan musyawarah warga',
							status: 'blocked',
							source: 'system',
							locked_fields: [],
							evidence_required: true,
							description: 'Membutuhkan quorum minimal 30% warga RT 05'
						}
					]
				},
				{
					phase_id: 'phase-3',
					title: 'Pelaksanaan Perbaikan',
					objective: 'Eksekusi perbaikan jalan dengan pengawasan warga',
					status: 'planned',
					source: 'ai',
					locked_fields: [],
					checkpoints: [
						{
							checkpoint_id: 'cp-7',
							title: 'Kontrak dengan tukang',
							status: 'planned',
							source: 'ai',
							locked_fields: []
						},
						{
							checkpoint_id: 'cp-8',
							title: 'Perbaikan jalan selesai',
							status: 'planned',
							source: 'ai',
							locked_fields: [],
							evidence_required: true
						},
						{
							checkpoint_id: 'cp-9',
							title: 'Laporan akhir ke kelurahan',
							status: 'planned',
							source: 'ai',
							locked_fields: []
						}
					]
				},
				{
					phase_id: 'phase-4',
					title: 'Pengawasan Kualitas',
					objective: 'Warga mengawasi hasil perbaikan selama 30 hari pertama',
					status: 'planned',
					source: 'ai',
					locked_fields: [],
					checkpoints: [
						{
							checkpoint_id: 'cp-10',
							title: 'Inspeksi mingguan oleh tim warga',
							status: 'planned',
							source: 'ai',
							locked_fields: []
						},
						{
							checkpoint_id: 'cp-11',
							title: 'Dokumentasi foto sebelum dan sesudah',
							status: 'planned',
							source: 'ai',
							locked_fields: [],
							evidence_required: true
						}
					]
				},
				{
					phase_id: 'phase-5',
					title: 'Pelaporan & Pertanggungjawaban',
					objective: 'Laporan keuangan dan hasil akhir ke seluruh warga',
					status: 'planned',
					source: 'ai',
					locked_fields: [],
					checkpoints: [
						{
							checkpoint_id: 'cp-12',
							title: 'Laporan keuangan lengkap',
							status: 'planned',
							source: 'ai',
							locked_fields: []
						},
						{
							checkpoint_id: 'cp-13',
							title: 'Musyawarah tutup proyek',
							status: 'planned',
							source: 'ai',
							locked_fields: [],
							evidence_required: true
						}
					]
				},
				{
					phase_id: 'phase-6',
					title: 'Perawatan Berkelanjutan',
					objective: 'Rencana perawatan rutin agar jalan tetap terjaga',
					status: 'planned',
					source: 'ai',
					locked_fields: [],
					checkpoints: [
						{
							checkpoint_id: 'cp-14',
							title: 'Jadwal perawatan triwulan',
							status: 'planned',
							source: 'ai',
							locked_fields: []
						},
						{
							checkpoint_id: 'cp-15',
							title: 'Dana cadangan perawatan disisihkan',
							status: 'planned',
							source: 'ai',
							locked_fields: []
						}
					]
				}
			]
		},
		{
			branch_id: 'branch-alt',
			label: 'Jalur Alternatif (Eskalasi)',
			parent_checkpoint_id: 'cp-6',
			phases: [
				{
					phase_id: 'phase-alt-1',
					title: 'Eskalasi ke DPRD',
					objective: 'Jika kelurahan tidak merespons dalam 14 hari, eskalasi ke tingkat kota',
					status: 'planned',
					source: 'ai',
					locked_fields: [],
					checkpoints: [
						{
							checkpoint_id: 'cp-alt-1',
							title: 'Surat resmi ke anggota DPRD dapil',
							status: 'planned',
							source: 'ai',
							locked_fields: [],
							description: 'Lampirkan semua bukti yang sudah terkumpul'
						},
						{
							checkpoint_id: 'cp-alt-2',
							title: 'Liputan media lokal',
							status: 'planned',
							source: 'ai',
							locked_fields: [],
							description: 'Hubungi redaksi media lokal untuk peliputan'
						}
					]
				}
			]
		}
	]
};

export const mockPathPlanEnvelope: PathPlanEnvelope = {
	path_plan: mockPathPlan,
	model_id: 'claude-sonnet-4-20250514',
	prompt_version: 'v2.1',
	generated_at: new Date().toISOString()
};
