import type { TriageService } from '../types';
import type { TriageResult, ContextBarState, CardEnrichment, TriageBudget, TrajectoryType, KelolaPayload } from '$lib/types';
import { mockPathPlan } from '$lib/fixtures';

const delay = (ms: number = 200) => new Promise<void>((resolve) => setTimeout(resolve, ms));

// ---------------------------------------------------------------------------
// Trajectory detection — maps primer text to trajectory + route
// ---------------------------------------------------------------------------

interface DetectedTrajectory {
	trajectory: TrajectoryType | null;
	route: TriageResult['route'];
}

/** Simple keyword-based trajectory detection from primer text. */
function detectTrajectory(primer: string): DetectedTrajectory {
	const p = primer.toLowerCase();

	// Kelola — group management (no trajectory)
	if (p.includes('kelola') || p.includes('kelompok') || p.includes('mengatur kelompok') || p.includes('manage a group'))
		return { trajectory: null, route: 'kelola' };

	// Siaga — emergency alert
	if (p.includes('siaga') || p.includes('darurat') || p.includes('bahaya') || p.includes('peringatan') || p.includes('emergency'))
		return { trajectory: 'siaga', route: 'siaga' };

	// Catat — data/documentation
	if (p.includes('catat') || p.includes('data') || p.includes('dokumentasi') || p.includes('fakta') || p.includes('bukti') || p.includes('document'))
		return { trajectory: 'data', route: 'catatan_komunitas' };

	// Musyawarah — deliberation
	if (p.includes('musyawarah') || p.includes('diskusi') || p.includes('keputusan') || p.includes('usul') || p.includes('discussion'))
		return { trajectory: 'mufakat', route: 'komunitas' };

	// Pantau — monitoring
	if (p.includes('pantau') || p.includes('awasi') || p.includes('mengawasi') || p.includes('monitor'))
		return { trajectory: 'pantau', route: 'komunitas' };

	// Bantuan — help request
	if (p.includes('bantuan') || p.includes('butuh') || p.includes('pertolongan') || p.includes('need help'))
		return { trajectory: 'bantuan', route: 'komunitas' };

	// Rayakan — celebration
	if (p.includes('rayakan') || p.includes('kabar baik') || p.includes('capai') || p.includes('celebrate') || p.includes('good news'))
		return { trajectory: 'pencapaian', route: 'komunitas' };

	// Program — recurring activity
	if (p.includes('program') || p.includes('jadwal') || p.includes('kegiatan rutin') || p.includes('organize'))
		return { trajectory: 'program', route: 'komunitas' };

	// Masalah — problem/action (default for content trajectories)
	if (p.includes('masalah') || p.includes('rusak') || p.includes('keluhan') || p.includes('problem'))
		return { trajectory: 'aksi', route: 'komunitas' };

	// Fallback
	return { trajectory: 'aksi', route: 'komunitas' };
}

// ---------------------------------------------------------------------------
// Per-trajectory card enrichments
// ---------------------------------------------------------------------------

const TRAJECTORY_ENRICHMENTS: Record<TrajectoryType, CardEnrichment> = {
	aksi: {
		icon: 'construction',
		trajectory_type: 'aksi',
		title: 'Jalan Berlubang Jl. Mawar, 30 KK Terdampak',
		hook_line: '3 bulan tanpa respons — warga turun tangan sendiri',
		pull_quote: 'Sudah lapor ke RT tapi belum ada tindakan',
		body: 'Warga Jl. Mawar melaporkan jalan rusak parah selama 3 bulan tanpa tindakan dari RT. Sekitar 30 KK terdampak. Warga siap gotong royong perbaiki sendiri jika ada dana.',
		sentiment: 'hopeful',
		intensity: 3,
		entity_tags: [
			{ label: 'Jl. Mawar', entity_type: 'lingkungan', confidence: 0.95 },
			{ label: 'Infrastruktur', entity_type: 'topik', confidence: 0.9 }
		],
		signal_labels: {
			saksi: { label: 'Saya Saksi', desc: 'Kamu melihat atau mengalami sendiri' },
			perlu_dicek: { label: 'Perlu Dicek', desc: 'Informasi perlu diverifikasi' }
		}
	},
	advokasi: {
		icon: 'megaphone',
		trajectory_type: 'advokasi',
		title: 'Advokasi Hak Air Bersih Kampung Melayu',
		hook_line: 'Warga kampung berjuang untuk akses air bersih',
		body: 'Komunitas Kampung Melayu mendesak pemerintah daerah untuk menyediakan akses air bersih yang layak.',
		sentiment: 'hopeful',
		intensity: 4,
		entity_tags: [
			{ label: 'Kampung Melayu', entity_type: 'lingkungan', confidence: 0.92 }
		]
	},
	mufakat: {
		icon: 'users',
		trajectory_type: 'mufakat',
		title: 'Usulan Taman Bermain Anak di Lahan Kosong RT 03',
		hook_line: 'Warga usulkan pemanfaatan lahan kosong untuk anak-anak',
		pull_quote: 'Anak-anak butuh ruang bermain yang aman',
		body: 'Warga RT 03 mengusulkan pembangunan taman bermain anak di lahan kosong samping masjid. Perlu musyawarah untuk memutuskan desain dan pendanaan.',
		sentiment: 'curious',
		intensity: 2,
		entity_tags: [
			{ label: 'RT 03', entity_type: 'lingkungan', confidence: 0.88 },
			{ label: 'Fasilitas Umum', entity_type: 'topik', confidence: 0.85 }
		]
	},
	mediasi: {
		icon: 'scale',
		trajectory_type: 'mediasi',
		title: 'Mediasi Sengketa Batas Lahan Antar Warga',
		hook_line: 'Dua warga berselisih soal batas tanah',
		body: 'Perlu mediasi untuk menyelesaikan sengketa batas tanah antara warga di RT 07.',
		sentiment: 'curious',
		intensity: 3,
		entity_tags: [
			{ label: 'RT 07', entity_type: 'lingkungan', confidence: 0.87 }
		]
	},
	pantau: {
		icon: 'eye',
		trajectory_type: 'pantau',
		title: 'Pemantauan Dana Desa Tahap II — Rp 120 Juta',
		hook_line: 'Warga pantau realisasi anggaran pembangunan jalan',
		pull_quote: 'Dana sudah cair tapi progress minim',
		body: 'Pemantauan realisasi Dana Desa tahap II sebesar Rp 120 juta untuk perbaikan jalan RT 05-07. Dana sudah cair sejak 2 bulan lalu tapi progress fisik baru 30%.',
		sentiment: 'curious',
		intensity: 3,
		entity_tags: [
			{ label: 'Dana Desa', entity_type: 'topik', confidence: 0.93 },
			{ label: 'RT 05-07', entity_type: 'lingkungan', confidence: 0.89 }
		]
	},
	data: {
		icon: 'file-text',
		trajectory_type: 'data',
		title: 'Harga Cabai Rawit di Pasar Minggu — Rp 85.000/kg',
		hook_line: 'Lonjakan harga cabai tertinggi bulan ini',
		body: 'Pencatatan harga cabai rawit di Pasar Minggu mencapai Rp 85.000/kg, naik 40% dari minggu lalu. Data ini penting untuk monitoring inflasi pangan lokal.',
		sentiment: 'curious',
		intensity: 1,
		entity_tags: [
			{ label: 'Pasar Minggu', entity_type: 'lingkungan', confidence: 0.91 },
			{ label: 'Harga Pangan', entity_type: 'topik', confidence: 0.95 }
		]
	},
	vault: {
		icon: 'lock',
		trajectory_type: 'vault',
		title: 'Catatan Rahasia — Laporan Internal',
		hook_line: 'Dokumen tersegel untuk arsip pribadi',
		body: 'Catatan pribadi yang disimpan dengan enkripsi dan tidak dapat diakses publik.',
		sentiment: 'sad',
		intensity: 1,
		entity_tags: []
	},
	bantuan: {
		icon: 'heart',
		trajectory_type: 'bantuan',
		title: 'Butuh Bantuan Hukum — Sengketa Tanah Warisan',
		hook_line: 'Warga cari pendamping hukum pro-bono',
		pull_quote: 'Tidak mampu bayar pengacara tapi hak waris terancam',
		body: 'Seorang warga membutuhkan bantuan hukum pro-bono untuk menyelesaikan sengketa tanah warisan keluarga. Kasus sudah berjalan 6 bulan tanpa kemajuan.',
		sentiment: 'sad',
		intensity: 3,
		entity_tags: [
			{ label: 'Hukum', entity_type: 'topik', confidence: 0.92 },
			{ label: 'Bantuan Pro-bono', entity_type: 'topik', confidence: 0.88 }
		]
	},
	pencapaian: {
		icon: 'trophy',
		trajectory_type: 'pencapaian',
		title: 'Jalan Berhasil Diperbaiki! Gotong Royong 2 Hari',
		hook_line: 'Warga buktikan kekuatan kolaborasi',
		pull_quote: 'Kalau bersama, semua bisa!',
		body: 'Setelah 2 hari gotong royong, jalan di Jl. Mawar berhasil diperbaiki oleh 45 warga. Total biaya Rp 3.2 juta dari iuran sukarela.',
		sentiment: 'celebratory',
		intensity: 5,
		entity_tags: [
			{ label: 'Jl. Mawar', entity_type: 'lingkungan', confidence: 0.95 },
			{ label: 'Gotong Royong', entity_type: 'topik', confidence: 0.93 }
		]
	},
	siaga: {
		icon: 'siren',
		trajectory_type: 'siaga',
		title: 'Peringatan Banjir — Sungai Ciliwung Siaga Merah',
		hook_line: 'Ketinggian air naik 2 meter dalam 6 jam',
		pull_quote: 'Segera evakuasi barang berharga ke lantai atas!',
		body: 'BMKG mengeluarkan peringatan banjir untuk kawasan bantaran Sungai Ciliwung. Ketinggian air sudah mencapai 3.5 meter (siaga merah). Warga diminta bersiap evakuasi.',
		sentiment: 'urgent',
		intensity: 5,
		entity_tags: [
			{ label: 'Sungai Ciliwung', entity_type: 'lingkungan', confidence: 0.97 },
			{ label: 'Bencana', entity_type: 'topik', confidence: 0.96 }
		]
	},
	program: {
		icon: 'calendar',
		trajectory_type: 'program',
		title: 'Jadwal Ronda RT 05 — Periode Februari 2026',
		hook_line: 'Rotasi jaga malam 4 shift per minggu',
		body: 'Jadwal ronda malam RT 05 untuk bulan Februari 2026. 4 shift per minggu, masing-masing 3 orang. Koordinator: Pak Budi.',
		sentiment: 'hopeful',
		intensity: 1,
		entity_tags: [
			{ label: 'RT 05', entity_type: 'lingkungan', confidence: 0.92 },
			{ label: 'Keamanan', entity_type: 'topik', confidence: 0.87 }
		]
	}
};

// ---------------------------------------------------------------------------
// Trajectory → triage behavior config
// ---------------------------------------------------------------------------

/** Trajectories that produce lifecycle witnesses (with proposed_plan). */
const WITNESS_TRAJECTORIES: TrajectoryType[] = [
	'aksi', 'advokasi', 'pantau', 'mufakat', 'mediasi', 'program'
];

/** Terminal bar_state per route. */
function terminalBarState(route: TriageResult['route']): ContextBarState {
	if (route === 'siaga') return 'siaga-ready';
	if (route === 'vault') return 'vault-ready';
	return 'ready';
}

// ---------------------------------------------------------------------------
// Budget simulation
// ---------------------------------------------------------------------------

/** Simulate token budget — standard trajectory for Contributor tier (6000 tokens). */
function mockBudget(turn: number, totalTokens: number = 6000): TriageBudget {
	const tokensPerTurn = 800 + Math.floor(Math.random() * 400);
	const used = Math.min(turn * tokensPerTurn, totalTokens);
	const remaining = totalTokens - used;
	return {
		total_tokens: totalTokens,
		used_tokens: used,
		remaining_tokens: remaining,
		budget_pct: used / totalTokens,
		can_continue: remaining > 0 && turn < 8,
		turn_count: turn,
		max_turns: 8
	};
}

// ---------------------------------------------------------------------------
// Mock Kelola response
// ---------------------------------------------------------------------------

const MOCK_KELOLA_RESULT: KelolaPayload = {
	action: 'create',
	group_detail: {
		name: 'Karang Taruna RT 05',
		description: 'Kelompok pemuda untuk kegiatan sosial dan pembangunan di lingkungan RT 05.',
		join_policy: 'terbuka',
		entity_type: 'kelompok'
	}
};

// ---------------------------------------------------------------------------
// Mock Triage Service
// ---------------------------------------------------------------------------

export class MockTriageService implements TriageService {
	private currentStep = 0;
	private detected: DetectedTrajectory = { trajectory: 'aksi', route: 'komunitas' };

	async startTriage(content: string, _attachments?: File[]): Promise<TriageResult> {
		await delay(500);
		this.currentStep = 1;
		this.detected = detectTrajectory(content);

		// Kelola path — ask group name on first step
		if (this.detected.route === 'kelola') {
			return {
				bar_state: 'probing',
				route: 'kelola',
				confidence: { score: 0.5, label: 'Kelola Kelompok...' },
				budget: mockBudget(1)
			};
		}

		return {
			bar_state: 'probing',
			route: this.detected.route,
			trajectory_type: this.detected.trajectory ?? undefined,
			confidence: { score: 0.4, label: 'Menganalisis...' },
			budget: mockBudget(1)
		};
	}

	async updateTriage(sessionId: string, answer: string, _attachments?: File[]): Promise<TriageResult> {
		await delay(400);
		this.currentStep = Math.min(this.currentStep + 1, 3);

		const { trajectory, route } = this.detected;

		// ── Kelola path ─────────────────────────────────────────────
		if (route === 'kelola') {
			if (this.currentStep === 2) {
				// Step 2 — leaning, ask description + policy
				return {
					bar_state: 'leaning',
					route: 'kelola',
					confidence: { score: 0.7, label: 'Kelola Kelompok · 70%' },
					budget: mockBudget(this.currentStep + 1)
				};
			}
			// Step 3+ — ready with kelola result
			return {
				bar_state: 'ready',
				route: 'kelola',
				confidence: { score: 0.95, label: 'Kelola Kelompok · 95%' },
				budget: mockBudget(this.currentStep + 1),
				kelola_result: MOCK_KELOLA_RESULT
			};
		}

		// ── Content trajectories ────────────────────────────────────
		if (this.currentStep >= 3) {
			// Terminal state
			const barState = terminalBarState(route);
			const enrichment = trajectory ? TRAJECTORY_ENRICHMENTS[trajectory] : TRAJECTORY_ENRICHMENTS.aksi;
			const isWitness = trajectory && WITNESS_TRAJECTORIES.includes(trajectory);

			const confidenceLabel = trajectory
				? `${enrichment.title.split(',')[0]} · 92%`
				: 'Aksi Bersama · 92%';

			return {
				bar_state: barState,
				route,
				track_hint: 'tuntaskan',
				seed_hint: 'Keresahan',
				trajectory_type: trajectory ?? undefined,
				card_enrichment: enrichment,
				confidence: { score: 0.92, label: confidenceLabel },
				budget: mockBudget(this.currentStep + 1),
				proposed_plan: isWitness ? { ...mockPathPlan, title: enrichment.title } : undefined
			};
		}

		// Intermediate state — leaning
		return {
			bar_state: 'leaning',
			route,
			track_hint: 'tuntaskan',
			trajectory_type: trajectory ?? undefined,
			confidence: {
				score: 0.4 + this.currentStep * 0.2,
				label: `Menganalisis... ${Math.round((0.4 + this.currentStep * 0.2) * 100)}%`
			},
			budget: mockBudget(this.currentStep + 1)
		};
	}
}
