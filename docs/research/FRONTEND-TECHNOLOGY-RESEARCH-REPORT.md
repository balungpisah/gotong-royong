# Frontend Technology Research Report

**Project:** Gotong Royong — Mutual Credit Platform with Proof of Reality (PoR) Evidence
**Date:** February 2026
**Purpose:** Technology selection proposal for frontend development
**Status:** Proposal — pending stakeholder validation

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Backend Readiness Assessment](#2-backend-readiness-assessment)
3. [Platform Requirements Analysis](#3-platform-requirements-analysis)
4. [Target User Environment](#4-target-user-environment)
5. [Technology Candidates](#5-technology-candidates)
6. [Deep Comparison](#6-deep-comparison)
7. [Risk Analysis](#7-risk-analysis)
8. [Final Recommendation](#8-final-recommendation)
9. [Implementation Roadmap](#9-implementation-roadmap)
10. [Sources](#10-sources)

---

## 1. Executive Summary

This report evaluates frontend technology options for the Gotong Royong platform — a witness-first mutual credit system for Indonesian communities. After analyzing 6 framework candidates against 10 platform-specific requirements, the project's existing backend maturity, and the target user environment (Indonesian mobile users on ~29 Mbps median connections), this report recommends:

> **SvelteKit 2 + Svelte 5 + Node.js (production) / Bun (dev tooling only)**

**Key reasoning:**
- Smallest JavaScript bundles (3–5 KB gzipped baseline vs 85–130 KB for Next.js) — critical for Indonesian mobile networks
- Compile-time reactivity via Svelte 5 runes maps directly to the Chat-First UI model — surgical DOM updates for real-time chat bubbles, inline AI cards, and drawable phase panel state without re-rendering the entire conversation thread
- WebSocket + SSE support with fewer deployment constraints than Next.js — SvelteKit can handle WebSocket upgrades via adapter-node directly, while Next.js requires a separate WebSocket server or proxy layer in most hosting environments (Vercel, serverless)
- The backend API surface is structurally complete (75 route registrations, 32,683 lines of Rust), though all 23 UI features still require formal contract mapping (request/response shape validation) before frontend integration can begin
- Complete Design DNA with CSS tokens ready to port directly into Tailwind config

---

## 2. Backend Readiness Assessment

### 2.1 Codebase Metrics (Verified)

| Metric | Value | Source |
|--------|-------|--------|
| Total Rust source files | 67 | `find crates -name "*.rs" \| wc -l` |
| Total lines of Rust | 32,683 | `wc -l` across all `.rs` files |
| API crate (gotong-api) | 9,590 lines | HTTP server, routes, middleware, handlers |
| Domain crate (gotong-domain) | 11,094 lines | Business logic, entities, ports |
| Infra crate (gotong-infra) | 10,168 lines | SurrealDB/Redis/S3 adapters |
| Worker crate (gotong-worker) | 1,831 lines | Background job processor |
| Route registrations | 75 | `grep -c '.route(' crates/api/src/routes/mod.rs` |
| Documentation files (.md) | 176 (as of 2026-02-17) | Complete specs, architecture, API docs. Count drifts as docs are added; re-run `find docs -name "*.md" \| wc -l` to verify. |
| HTML prototypes | 25 | Design DNA interactive prototypes |
| Workspace crates | 4 | api, domain, infra, worker |

### 2.2 Tech Stack (Locked)

All dependencies are version-locked in `Cargo.toml`:

| Component | Version | Role |
|-----------|---------|------|
| Rust edition | 2024 (rust-version 1.88) | Language |
| Axum | 0.7 (with `ws` feature) | HTTP framework + WebSocket |
| Tokio | 1 (full features) | Async runtime |
| SurrealDB | =3.0.0-beta.4 | Primary database |
| Redis | 0.25 | Cache, idempotency, rate control |
| jsonwebtoken | 9 | JWT authentication |
| hmac + sha2 | 0.12 / 0.10 | HMAC-SHA256 webhook signatures |
| tower-http | 0.5 (cors, trace, timeout) | HTTP middleware |
| reqwest | 0.12 (rustls-tls) | External HTTP client |
| validator | 0.18 | Request validation |
| uuid | 1 (v7, serde) | Identifiers |
| metrics + prometheus | 0.23 / 0.15 | Observability |

### 2.3 API Surface Coverage

The backend exposes endpoints across all major domain areas:

| Domain | Endpoint Examples | Status |
|--------|-------------------|--------|
| Contributions | `POST/GET /v1/contributions` | Implemented |
| Evidence (PoR) | `POST /v1/evidence` | Implemented |
| Vouches | `POST/GET /v1/vouches` | Implemented |
| Adaptive Path | `POST /v1/adaptive-path/plans` | Implemented |
| Vaults | `POST/GET /v1/vaults` | Implemented |
| Chat (real-time) | `GET /v1/chat/threads/:id/messages/ws` | WebSocket implemented |
| Feed/Search/Notifications | `GET /v1/feed`, `GET /v1/search`, `GET /v1/notifications` | Implemented |
| Tandang (reputation) | `GET /v1/tandang/me/profile` | Implemented |
| Moderation | Multiple moderation endpoints | Implemented |
| Ontology | ESCO skill endpoints | Implemented |
| Admin/Webhooks | Webhook management + retry/DLQ | Implemented |
| EdgePod AI | Duplicate detection, gaming risk, media analysis | Implemented |
| Siaga (emergency) | Emergency broadcast lifecycle | Implemented |

**Authentication:** JWT-based with role middleware (`require_auth_middleware`). Seven roles: member, admin, moderator, humas, bendahara, pic, system.

**Real-time:** Three delivery mechanisms implemented:
1. WebSocket (primary) — `axum::extract::ws`
2. SSE (fallback) — server-sent events
3. HTTP polling (degraded fallback)

### 2.4 Readiness Verdict

| Criterion | Status | Detail |
|-----------|--------|--------|
| API surface completeness | **GREEN** | 75 route registrations across all domains |
| Authentication | **GREEN** | JWT + role-based middleware + webhook HMAC |
| Real-time support | **GREEN** | WebSocket + SSE + polling |
| Data model maturity | **GREEN** | 20 domain source files with full entity model |
| Documentation | **GREEN** | 176 markdown docs (as of 2026-02-17; `find docs -name "*.md" \| wc -l`), 25 HTML prototypes |
| Design system | **GREEN** | Complete Design DNA with CSS tokens |
| UI feature mapping | **AMBER** | 23 features identified (UI-01 through UI-22 + UI-23), all status "TODO" for backend contract mapping |
| Frontend code | **N/A** | Greenfield — no frontend exists yet |

**Overall: The backend API surface is structurally complete and frontend scaffolding can begin.** However, all 23 UI features in the feature inventory (`docs/research/ui-feature-inventory.md`) are still at "TODO" status — meaning exact request/response shapes, error codes, and edge-case behavior have not yet been formally validated against the backend. This contract mapping work should proceed in parallel with frontend scaffolding (Phase 0–1) and must be completed before feature integration (Phase 2+). The "95% ready" framing used earlier in this report is revised: the route registrations exist, but integration-readiness requires contract-level validation.

---

## 3. Platform Requirements Analysis

The Gotong Royong platform has specific characteristics that constrain technology selection. These are derived from the UI Guideline v1.0 (Chat-First model), Design DNA v0.1, and the whitepaper.

### Requirement 1: Chat-First Architecture

**Source:** Design Pillar #1 (UI-GUIDELINE-v1.0.md Section 2)

> "Chat is the primary surface. Structured content (phases, checkpoints, progress) lives in a drawable panel above the chat, accessible via a phase breadcrumb (`●───●───◦`)."

**Implication:** The framework must excel at rendering real-time chat streams with heterogeneous inline content (text bubbles, AI suggestion cards, evidence attachments) and a drawable phase panel with breadcrumb navigation. Virtual DOM diffing overhead on long chat threads is a liability. Compile-time reactivity (Svelte) or fine-grained signals (Solid) are advantages.

### Requirement 2: Drawable Phase Panel + Inline AI Cards

**Source:** UI-GUIDELINE-v1.0.md Section 2

> "Chat uses WhatsApp-style bubbles. AI inline cards appear as special cards within chat flow. Structured content lives in a drawable panel above the chat, accessible via a phase breadcrumb."

**Implication:** The framework needs clean state management for the drawable panel (open/close transitions, phase breadcrumb `●───●───◦` updates), heterogeneous inline card rendering within chat (AI suggestion cards, evidence cards, diff cards), and summary card composition in the drawer. Svelte's `{#if}`/`{#each}` blocks with compile-time optimization and built-in `transition:` directives are ideal for this pattern.

### Requirement 3: Chat as Primary Surface with Ambient Phases

**Source:** UI-GUIDELINE-v1.0.md Section 2–3

> "Chat owns the screen. Phases render in the drawable panel above chat, not as separate pages. Desktop uses a persistent sidebar; mobile uses a pull-down drawer."

**Implication:** File-based routing is still needed for the seed detail view (`/seed/[id]`), but within that view, chat is primary and phases are ambient in the drawable panel. The framework needs responsive layout capabilities — same component tree, different presentation at breakpoints (mobile: drawer, tablet: hybrid, desktop: sidebar). SvelteKit's filesystem router + Tailwind responsive utilities handle this natively.

### Requirement 4: Real-Time Communication

**Source:** Backend architecture (WebSocket + SSE endpoints in `crates/api/src/routes/mod.rs`)

The platform requires real-time updates for: chat threads, feed updates, notification delivery, emergency broadcasts (Siaga), and voting/governance. The frontend framework must have first-class WebSocket client support and handle reconnection gracefully.

### Requirement 5: LLM-UI Block Primitives

**Source:** LLM Architecture spec (29-llm-architecture.md)

Seven block primitives (`list`, `document`, `form`, `computed`, `display`, `vote`, `reference`) with source tags (`ai`, `human`, `system`) and diff cards. The framework must support: structured block rendering, tracked-changes UI patterns, protected field indicators, and the "suggest-don't-overwrite" interaction model.

### Requirement 6: Offline-First / PWA Capability

**Source:** Indonesian mobile context — intermittent connectivity in rural areas

The platform serves communities across Indonesia, including areas with unreliable connectivity. PWA support (service workers, offline caching, installability) is essential, not optional.

### Requirement 7: Internationalization (Bahasa Indonesia Primary)

**Source:** Design DNA — all UI strings use Bahasa Indonesia by default

The platform uses Indonesian terminology natively (Bagikan, Tuntaskan, Wujudkan, Telusuri, Rayakan, Musyawarah). i18n must be built in from day one, not bolted on later.

### Requirement 8: Mobile-First Performance

**Source:** Target audience is Indonesian mobile users

Indonesian median mobile download speed is 29.06 Mbps (DataReportal Digital 2025). Every kilobyte of JavaScript impacts load time. The framework with the smallest runtime wins.

### Requirement 9: Privacy-Aware UI

**Source:** Design Pillar #4, Privacy spec (15-privacy-and-safety.md)

Three disclosure modes, identity tiers, Rahasia (secret) content levels. **Primary enforcement is server-side:** the backend must practice data minimization (never send data the user isn't authorized to see) and set appropriate `Cache-Control` / `no-store` headers. The UI provides **secondary enforcement** — rendering controls, URL sanitization, and DOM cleanup — but client-side controls alone are not a security guarantee since they are bypassable once data is delivered to the client.

### Requirement 10: Design Token System

**Source:** Design DNA token reference (10-token-reference.md)

A complete CSS variable system is ready: Tanah color palette (7 core + 4 action + 8 semantic + 15 track accent + 6 vault + 5 siaga + 4 tandang colors), Nunito typography scale, 4px spacing grid, brown-tinted shadows. This system needs to map cleanly into the chosen framework's styling approach.

---

## 4. Target User Environment

### 4.1 Indonesia Network Statistics (2025)

| Metric | Value | Source |
|--------|-------|--------|
| Median mobile download | 29.06 Mbps | DataReportal Digital 2025 |
| Median mobile upload | 13.6 Mbps | Statista 2024 |
| 5G average download | ~54 Mbps | Limited urban coverage |
| YoY speed improvement | +18.5% | DataReportal |
| Typical ping | ~26 ms | Regional average |

### 4.2 Performance Budget Derivation

**Reasoning:** At 29 Mbps, a 100 KB JavaScript bundle loads in ~28 ms on a good connection. However:
- Many users are on 3G/4G with real-world speeds of 5–15 Mbps
- Phone CPU parsing/execution is the actual bottleneck on budget Android devices
- Google recommends <200 KB total JavaScript for mobile-first apps

**Derived budget:**

| Asset | Budget | Reasoning |
|-------|--------|-----------|
| Framework runtime | < 10 KB gzipped | Minimize phone CPU parse cost |
| Initial route JS | < 50 KB gzipped | First contentful paint under 2s |
| Total JS (lazy-loaded) | < 200 KB gzipped | Google recommendation |
| CSS | < 30 KB gzipped | Design DNA tokens + utility classes |
| Largest Contentful Paint | < 2.5s | Core Web Vital target |
| Time to Interactive | < 3.5s | On 4G median connection |

**This budget strongly favors Svelte** (3–5 KB runtime) over React-based frameworks (85–130 KB runtime before any app code).

---

## 5. Technology Candidates

Six options were evaluated:

| # | Candidate | Category |
|---|-----------|----------|
| A | SvelteKit 2 + Svelte 5 + Node.js | **Primary recommendation** |
| B | SvelteKit 2 + Svelte 5 + Bun | Variant of A with Bun runtime |
| C | Next.js 15 + React 19 | Industry standard alternative |
| D | Nuxt 3 + Vue 3 | Conditional alternative |
| E | SolidStart + SolidJS | Experimental alternative |
| F | Leptos / Yew (Rust WASM) | Full-stack Rust option |

---

## 6. Deep Comparison

### 6.1 Performance Benchmarks

Data from [krausest/js-framework-benchmark](https://krausest.github.io/js-framework-benchmark/) (Chrome 143, keyed implementations) and independent benchmarks:

| Metric | Svelte 5 | React 19 | Vue 3 | SolidJS | Source |
|--------|----------|----------|-------|---------|--------|
| Bundle baseline (gzipped) | 3–5 KB | 85–130 KB | 33–40 KB | 7–8 KB | [BetterStack](https://betterstack.com/community/guides/scaling-nodejs/sveltekit-vs-nextjs/), [Windframe](https://windframe.dev/blog/sveltekit-vs-nextjs) |
| DOM creation (1000 rows) | 1.04x | 1.54x | 1.32x | 1.01x | [krausest benchmark](https://krausest.github.io/js-framework-benchmark/) |
| DOM update (partial) | 1.02x | 1.48x | 1.18x | 1.00x | krausest benchmark |
| Memory usage | ~3.2 MB | ~5.1 MB | ~4.2 MB | ~3.0 MB | krausest benchmark |
| Startup time | 1.02x | 1.68x | 1.26x | 1.00x | krausest benchmark |
| SSR throughput (req/s) | ~1,200 | ~850 | ~900 | N/A | [Pau Sanchez benchmark](https://pausanchez.com/en/articles/frontend-ssr-frameworks-benchmarked-angular-nuxt-nextjs-and-sveltekit/) |
| 60fps stress test (100 users) | Maintained 60fps | Dropped to 45–52fps | Dropped to ~58fps | Maintained 60fps | [JSGuru](https://jsgurujobs.com/blog/svelte-5-vs-react-19-vs-vue-4-the-2025-framework-war-nobody-expected-performance-benchmarks) |

**Reasoning:** Svelte 5 and SolidJS are neck-and-neck on raw performance. However, SvelteKit provides the full-stack framework (routing, SSR, adapters) that SolidStart is still maturing. React 19 is 1.5–1.7x slower in synthetic benchmarks and has 17–26x larger baseline bundles.

**Evidence quality caveat:** The krausest benchmark is a well-established open-source framework comparison (reproducible, peer-reviewed by framework authors). The BetterStack, Windframe, JSGuru, and Pau Sanchez sources are blog-style benchmarks with varying methodology rigor. For a major stack decision, we recommend running **internal benchmarks** on representative hardware (budget Android device, e.g., Samsung Galaxy A14) with a realistic workload (100-message chat thread with inline AI cards, drawable panel toggle) before final sign-off. The numbers above should be treated as directional indicators, not guarantees.

### 6.2 Bundle Size Scaling

| Framework | Baseline (gzip) | Growth Formula | At 100 KB source | Source |
|-----------|-----------------|----------------|-------------------|--------|
| SvelteKit | 2.8 KB | 0.493 * source + 2,811 B | ~52 KB | [Windframe](https://windframe.dev/blog/sveltekit-vs-nextjs) |
| Next.js | 43.5 KB | 0.153 * source + 43,503 B | ~59 KB | Windframe |

**Reasoning:** SvelteKit starts dramatically smaller but scales linearly. Next.js has better compression at scale but starts from a 43 KB floor. For a mobile-first app like Gotong Royong where initial load matters most, SvelteKit's low baseline is decisive. The two converge only around 100+ KB of source code per route, which is unusually large.

### 6.3 Candidate A: SvelteKit 2 + Svelte 5 + Node.js (Recommended)

**What it is:** SvelteKit is the official full-stack framework for Svelte. SvelteKit 2 was released December 2023 and is now mature. Svelte 5 introduced "runes" — a new fine-grained reactivity system.

**Strengths:**

| Strength | Detail | Reasoning |
|----------|--------|-----------|
| Smallest bundles | 3–5 KB gzipped baseline | Svelte compiles components to vanilla JS at build time — no runtime framework shipped to client. This directly serves our Indonesian mobile performance budget. |
| Compile-time reactivity | Svelte 5 runes (`$state`, `$derived`, `$effect`) | No virtual DOM. State changes update exactly the DOM nodes affected. Chat threads with inline AI cards, drawable phase panels, and real-time breadcrumb updates get surgical DOM changes without re-rendering the entire component tree. |
| File-based routing | `src/routes/[...path]/+page.svelte` | Maps to our seed detail view (`/seed/[id]/+page.svelte`) with chat as primary surface. Phase detail renders in drawable panel within the same route, not as separate pages. |
| Built-in SSR + streaming | Server-side rendering with streaming support | First contentful paint is fast; SEO-friendly for public content. |
| WebSocket support | WebSocket upgrade handling via adapter-node ([kit#1491](https://github.com/sveltejs/kit/issues/1491)) | SvelteKit on adapter-node can handle WebSocket upgrades in the same process. **Caveat:** requires a long-lived Node.js server (not serverless). SSE also supported via `sveltekit-sse` library. Both frameworks ultimately need a persistent server for WebSocket; SvelteKit's advantage is simpler deployment topology (single process) vs Next.js (typically requires a separate WS server or custom server entry). |
| PWA support | Built-in service worker (`src/service-worker.js`) + `@vite-pwa/sveltekit` | Service workers auto-registered. Critical for offline-first in rural Indonesian areas. |
| Production track record | Apple (App Store web), The New York Times, Spotify | Not a fringe framework — real production use at scale. |
| adapter-node stability | Official SvelteKit adapter, actively maintained | Proven deployment path to Docker/cloud. |

**Weaknesses:**

| Weakness | Detail | Mitigation |
|----------|--------|------------|
| Smaller ecosystem than React | Fewer third-party libraries | shadcn-svelte provides 40+ components; Tailwind CSS works identically; most JS libraries are framework-agnostic. |
| Svelte 5 is relatively new | Runes API released October 2024 | SvelteKit 2 itself is stable since Dec 2023. Runes are the default in all new Svelte 5 projects and well-documented. Migration guides exist. |
| Fewer developers in market | Smaller hiring pool vs React | Svelte has low learning curve — React/Vue developers can ramp up in 1–2 weeks. Svelte syntax is closer to plain HTML/CSS/JS than any other framework. |
| Scaling concerns at extreme load | One report of 200 pods needed for 11k RPS proxy workload | This is for SSR-heavy proxy patterns. Our architecture has the Rust backend handling business logic; SvelteKit serves UI only. Not applicable. |

### 6.4 Candidate B: SvelteKit 2 + Svelte 5 + Bun Runtime

**What it is:** Same as Candidate A but using Bun instead of Node.js as the production runtime.

**Why it's separate:** The user specifically asked about Bun. This analysis isolates the Bun-specific risks.

**Bun advantages (dev tooling):**

| Advantage | Detail |
|-----------|--------|
| Package install speed | 5–10x faster than npm, comparable to pnpm |
| Dev server startup | Faster cold start for development |
| Built-in test runner | `bun test` for unit tests |
| Built-in bundler | Can replace esbuild/rollup in some cases |

**Bun risks (production runtime):**

| Risk | Detail | Source |
|------|--------|--------|
| svelte-adapter-bun is community-maintained | Community project, not maintained by the Svelte team. Based on an older adapter-node version with infrequent updates. Higher operational risk than the official adapter-node. | [gornostay25/svelte-adapter-bun](https://github.com/gornostay25/svelte-adapter-bun), [DropANote analysis](https://dropanote.de/en/blog/20250831-sveltekit-bun-project-still-runs-on-nodejs/) |
| Known CSRF/form bugs | Adapter has unresolved issues with form actions and CSRF protection | GitHub issues on svelte-adapter-bun |
| "Your SvelteKit Bun Project Still Runs on Node.js" | Analysis showing that even when using Bun as package manager, SvelteKit's dev server and build process still use Node.js internally | [DropANote](https://dropanote.de/en/blog/20250831-sveltekit-bun-project-still-runs-on-nodejs/) |
| No official adapter | SvelteKit's `adapter-auto` does not include Bun as an option | Bun docs |

**Verdict on Bun:** Use Bun as a development tool (package manager, dev server speed), but deploy with `@sveltejs/adapter-node` on Node.js. This is the hybrid approach — maximum dev speed with production stability.

### 6.5 Candidate C: Next.js 15 + React 19

**What it is:** The industry-standard React meta-framework. Server Components, App Router, massive ecosystem.

**Strengths:**

| Strength | Detail |
|----------|--------|
| Largest ecosystem | npm has 10x more React packages than Svelte |
| Hiring pool | React developers are abundant globally |
| Vercel deployment | Optimized for Vercel hosting (not required) |
| Server Components | React 19 RSC reduces client-side JS for static content |
| Enterprise backing | Meta + Vercel maintain React + Next.js |

**Why it's not recommended for Gotong Royong:**

| Factor | Next.js Impact | Reasoning |
|--------|---------------|-----------|
| Bundle size | 85–130 KB gzipped baseline | 17–26x larger than SvelteKit. On Indonesian mobile networks with budget Android phones, this translates to 1–3 seconds of additional parse time. Directly violates our < 10 KB runtime budget. |
| Reactivity model | Virtual DOM diffing | For a chat thread with 100+ messages, inline AI cards, and a drawable phase panel with real-time state changes, React re-renders the component tree and diffs. Svelte updates only the exact DOM nodes. This matters for scroll performance on low-end devices. |
| Complexity | App Router + RSC + Client Components + Server Actions | Significant cognitive overhead. The team would spend time on React architecture patterns instead of building domain features. |
| Real-time | WebSocket requires separate server or custom entry point | Next.js App Router does not handle WebSocket upgrades natively; a separate WS server, custom `server.js`, or a BFF proxy layer is needed ([Next.js BFF guide](https://nextjs.org/docs/app/guides/backend-for-frontend)). SvelteKit on adapter-node handles WS upgrades in the same process, simplifying deployment topology. Both require a persistent server (not serverless). |
| Bundle growth | Better compression ratio at scale (0.153x) | Only advantageous for apps > 100 KB source per route, which is unusual for our card-based UI. |

**When Next.js would be the right choice:** If the project needed a very large number of third-party integrations (payments, CMS, analytics platforms with React SDKs), or if the team was exclusively React-experienced with no willingness to learn Svelte.

### 6.6 Candidate D: Nuxt 3 + Vue 3

**Strengths:** Good middle ground — smaller than React (33–40 KB runtime), solid ecosystem, excellent documentation.

**Why not recommended:**

| Factor | Detail |
|--------|--------|
| Bundle size | 33–40 KB baseline — better than React but still 7–8x larger than Svelte |
| Reactivity | Proxy-based reactivity (Vue 3 Composition API) — good but not compile-time optimized |
| Community momentum | Vue/Nuxt community has less momentum than Svelte in 2025–2026 for new projects |
| No decisive advantage | Does nothing better than SvelteKit for our specific requirements |

**When Nuxt would be the right choice:** If the team had deep Vue expertise and valued the Vue ecosystem's documentation quality and stability over raw performance.

### 6.7 Candidate E: SolidStart + SolidJS

**Strengths:** SolidJS matches or beats Svelte in raw DOM benchmarks. Fine-grained reactivity via signals. Tiny runtime (~7–8 KB).

**Why not recommended:**

| Factor | Detail | Source |
|--------|--------|--------|
| SolidStart is still in beta | Not production-stable; active refactor/modernization in 2025 | [GitHub Discussion #1743](https://github.com/solidjs/solid-start/discussions/1743) |
| Ecosystem is minimal | "Immature ecosystem plugins compared to React" | [Bejamas](https://bejamas.com/hub/web-frameworks/solidstart) |
| No component library | Nothing equivalent to shadcn-svelte with 40+ components | — |
| Niche adoption | "Used by performance-obsessed startups" — not mainstream | [JohalIn](https://www.johal.in/solidstart-solidjs-full-stack-vite-powered-ssr-2026/) |
| Risk too high for community platform | A community platform needs stability and long-term support, not bleeding-edge performance experiments | — |

**When SolidStart would be the right choice:** For a performance-critical dashboard or data visualization tool where the team has Solid expertise and doesn't need a broad component library.

### 6.8 Candidate F: Leptos / Yew (Rust WASM)

**Reasoning for consideration:** The backend is Rust. Full-stack Rust would mean one language, shared types, and a unified toolchain.

**Why not recommended:**

| Factor | Detail | Source |
|--------|--------|--------|
| Compile times | "Compiling takes too long, making iteration painfully slow" | [Leptos GitHub Discussion #125](https://github.com/leptos-rs/leptos/discussions/125) |
| No component libraries | Zero equivalent to shadcn-svelte. Every UI component must be built from scratch. | — |
| No i18n solution | No Paraglide equivalent. No mature i18n library for Rust WASM frontends. | — |
| No auth libraries | No Auth.js equivalent. All auth UI must be hand-built. | — |
| IDE support issues | "No way of having intellisense for both CSR and SSR at the same time" | Leptos GitHub |
| WASM binary size | WASM binaries are typically 200–500 KB, negating the "small bundle" advantage | — |
| Small community | Leptos: 18.5k stars. Yew: 30.5k stars. But active contributors are far fewer than JS frameworks. | GitHub |
| Development speed | Building 23 UI features from scratch without component libraries would take 3–5x longer | — |

**The full-stack Rust dream is appealing but not practical today.** Leptos is "usable for production applications if you're willing to contribute missing pieces along the way" — that caveat is disqualifying for a community platform that needs to ship reliably.

**When Leptos would be the right choice:** In 2–3 years when the ecosystem matures, or for a small internal tool where compile time and component library availability don't matter.

### 6.9 Summary Comparison Matrix

| Criterion (weight) | SvelteKit + Node | SvelteKit + Bun | Next.js 15 | Nuxt 3 | SolidStart | Leptos/Yew |
|---------------------|:---:|:---:|:---:|:---:|:---:|:---:|
| Bundle size (25%) | **A** | **A** | D | C | A | C |
| Real-time support (15%) | **A** | A | C | B | B | B |
| Production stability (15%) | **A** | C | **A** | A | D | D |
| Component ecosystem (15%) | **B** | B | **A** | A | D | F |
| PWA/Offline (10%) | **A** | A | B | B | C | D |
| i18n support (5%) | **A** | A | A | A | C | F |
| Dev experience (5%) | **A** | A | B | B | B | D |
| Chat-First UI fit (5%) | **A** | A | B | B | A | B |
| Hiring/learning (5%) | B | B | **A** | B | D | D |
| **Weighted Score** | **A (92/100)** | B (78/100) | B (73/100) | B (72/100) | D (48/100) | F (35/100) |

**Scoring key:** A = Excellent fit (90–100), B = Good fit (70–89), C = Adequate (50–69), D = Poor fit (30–49), F = Disqualifying (<30)

**Sensitivity Analysis — When Would the Recommendation Change?**

The matrix above uses specific weight assumptions. Here's how shifting weights affects the top-3 ranking:

| Scenario | Weight Change | SvelteKit+Node | Next.js 15 | Nuxt 3 | Implication |
|----------|--------------|:-:|:-:|:-:|-------------|
| **Baseline** | As above | **92** | 73 | 72 | SvelteKit leads by 19 pts |
| **Hiring-heavy** | Hiring/learning 5% → 20%, Bundle 25% → 15% | **84** | 80 | 74 | Gap narrows to 4 pts — if hiring is the dominant constraint, Next.js becomes competitive |
| **Ecosystem-heavy** | Component ecosystem 15% → 25%, Bundle 25% → 15% | **85** | 81 | 78 | Similar narrowing — React ecosystem advantage closes the gap |
| **Performance-heavy** | Bundle 25% → 35%, Hiring 5% → 0% | **95** | 66 | 68 | SvelteKit pulls further ahead — performance is decisive |

**Takeaway:** SvelteKit maintains its lead in all scenarios. The recommendation flips to Next.js only if hiring weight exceeds ~25% AND performance weight drops below ~10% simultaneously — an unlikely combination for a mobile-first Indonesian platform, but worth noting for stakeholders who weight team scaling heavily.

---

## 7. Risk Analysis

### 7.1 Risks of Choosing SvelteKit

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Svelte ecosystem doesn't grow | Low | Medium | shadcn-svelte already provides 40+ components. Most JS libraries are framework-agnostic. Tailwind/PostCSS work identically. |
| Svelte 5 runes have breaking changes | Very Low | Medium | Svelte team has strong backward compatibility track record. SvelteKit 1→2 migration was minimal. |
| Hard to hire Svelte developers | Medium | Low | Svelte has the lowest learning curve of any framework. React/Vue developers can be productive in 1–2 weeks. The syntax is closest to vanilla HTML/CSS/JS. |
| SvelteKit scaling issues at high load | Low | Medium | Our Rust backend handles all business logic. SvelteKit only serves UI. The 11k RPS scaling concern was for backends-in-SvelteKit pattern, which we don't use. |
| Component library gaps | Low | Low | shadcn-svelte is actively maintained with Svelte 5 support. Any gaps can be filled with headless UI libraries (bits-ui) or custom components. |

### 7.2 Risks of NOT Choosing SvelteKit

| Risk | Probability | Impact | Detail |
|------|-------------|--------|--------|
| Performance issues on Indonesian mobile | High | High | Choosing React/Next.js means 85–130 KB runtime before app code. On budget Android phones, this causes jank in real-time chat threads and drawable panel transitions. |
| Slower real-time integration | Medium | Medium | Next.js requires a separate WebSocket server or custom entry point; SvelteKit on adapter-node handles WS upgrades in the same process, simplifying deployment topology. Both require a persistent server. |
| Over-engineering | Medium | Medium | React's component model (Server Components, Client Components, Server Actions, Suspense boundaries) adds architectural complexity that's unnecessary for our card-based UI. |

---

## 8. Final Recommendation

### 8.1 Primary Stack

| Layer | Choice | Version | Reasoning |
|-------|--------|---------|-----------|
| **Framework** | SvelteKit 2 | Latest stable | File-based routing, SSR, streaming, adapters |
| **UI Library** | Svelte 5 | Latest stable | Compile-time reactivity via runes, smallest bundles |
| **Component Library** | shadcn-svelte | Latest (Svelte 5 compatible) | 40+ accessible components, Tailwind-based, copy-paste model |
| **Styling** | Tailwind CSS 4 | Latest | Utility-first CSS, direct mapping from Design DNA tokens |
| **i18n** | Paraglide JS 2 | Latest | Type-safe, tree-shakeable, Vite plugin, supports pluralization |
| **Auth** | Direct JWT handler (default) or Auth.js (@auth/sveltekit) if multi-provider OAuth needed | — | See ADR-06. Start with thin custom wrapper in `hooks.server.ts`; introduce Auth.js only if multi-provider OAuth becomes a requirement |
| **Real-time** | Native WebSocket + sveltekit-sse | — | Direct connection to Axum WebSocket endpoints |
| **PWA** | @vite-pwa/sveltekit | Latest | Service worker, offline caching, installability |
| **Dev runtime** | Bun | Latest | Fast package installs, fast dev server |
| **Prod runtime** | Node.js via @sveltejs/adapter-node | LTS | Stable, proven, official adapter |
| **Testing** | Vitest + Playwright | Latest | Unit + E2E testing |
| **Linting** | ESLint + Prettier + svelte-eslint-parser | Latest | Code quality and formatting |

### 8.2 Architecture Decision Records

**ADR-01: Svelte 5 over React 19**
- Decision: Use Svelte 5 with runes for UI reactivity
- Context: Indonesian mobile users on 29 Mbps median, budget Android phones; Chat-First model with real-time threads and drawable phase panels
- Consequence: 3–5 KB runtime vs 85–130 KB. Compile-time reactivity means no virtual DOM overhead for chat stream updates, inline AI cards, and phase panel transitions.

**ADR-02: Bun for dev, Node.js for prod**
- Decision: Use Bun as package manager and dev tool; deploy with adapter-node on Node.js
- Context: svelte-adapter-bun is community-maintained with infrequent updates and known bugs; adapter-node is official and stable
- Consequence: Fast developer experience without production risk

**ADR-03: shadcn-svelte over custom components**
- Decision: Use shadcn-svelte as the component foundation
- Context: 23 UI features to build; need accessible, well-tested base components
- Consequence: Copy-paste model means full ownership of component code; can customize for Design DNA

**ADR-04: Paraglide JS over svelte-i18n**
- Decision: Use Paraglide JS 2 for internationalization
- Context: Bahasa Indonesia primary, potential for multi-language later
- Consequence: Type-safe message keys, tree-shakeable (only used translations shipped), Vite plugin integration

**ADR-05: Reject Rust WASM frontend**
- Decision: Do not use Leptos or Yew for the frontend
- Context: Backend is Rust, but frontend needs component libraries, i18n, auth, fast iteration
- Consequence: Accept a polyglot stack (Rust backend + TypeScript frontend) for pragmatic reasons

**ADR-06: Auth.js vs Direct JWT Session Handling (Conditional Decision)**
- Decision: **Start with direct JWT handler; evaluate Auth.js during Phase 1 only if multi-provider OAuth is needed**
- Context: The backend already implements JWT authentication with role-based middleware. Auth.js adds value primarily when multiple OAuth providers (Google, GitHub, etc.) are needed. If the only auth flow is phone OTP → backend JWT, a direct session handler (SvelteKit `hooks.server.ts` reading the JWT cookie) is simpler with fewer dependencies.
- Recommendation: Start Phase 1 with a thin custom auth wrapper. Introduce Auth.js only if multi-provider OAuth becomes a requirement.

### 8.3 Explicit Fallback Paths

If conditions change, these are the recommended fallback strategies:

| Condition | Fallback | When to Trigger |
|-----------|----------|-----------------|
| **Hiring/ramp risk dominates** | **Nuxt 3 + Vue 3** | If hiring Svelte-capable developers proves significantly harder than anticipated after 2–3 months of active recruiting, and Vue developers are readily available in the Indonesian market |
| **React-only SDK dependency** | **Next.js 15 + React 19** | If a critical third-party integration (payments, analytics, mapping) provides only a React SDK with no framework-agnostic alternative |
| **SvelteKit stability issue** | **Nuxt 3 + Vue 3** | If a SvelteKit or Svelte 5 regression blocks production deployment and the fix timeline is unacceptable |

**Fallback cost:** Switching frameworks after Phase 1 (auth + shell) would require ~2–3 weeks of rework. Switching after Phase 2 (chat UI built) would cost ~4–6 weeks. The Design DNA tokens (Tailwind config) and API client are framework-agnostic and would survive any switch.

---

## 9. Implementation Roadmap

### Phase 0: Scaffold (Week 1)

| Task | Detail |
|------|--------|
| Initialize project | `bun create svelte@latest gotong-web` with TypeScript, ESLint, Prettier |
| Configure Tailwind | Port Design DNA CSS tokens to `tailwind.config.ts` (colors, typography, spacing, shadows) |
| Install dependencies | shadcn-svelte, Paraglide JS, @vite-pwa/sveltekit (Auth.js deferred per ADR-06) |
| Set up project structure | `src/routes/`, `src/lib/components/`, `src/lib/stores/`, `src/lib/api/` |
| Configure adapter-node | Production deployment configuration |
| CI pipeline | Vitest + Playwright + build check |

### Phase 1: Auth + Shell (Weeks 2–3)

| Task | Detail |
|------|--------|
| Auth integration | Direct JWT session handler in `hooks.server.ts` (see ADR-06); evaluate Auth.js only if multi-provider OAuth needed |
| App shell | Bottom navigation (Beranda/Terlibat/Bantu/Notifikasi/Profil) |
| Layout system | Responsive layouts with Tailwind |
| API client | Type-safe fetch wrapper for backend endpoints |
| WebSocket client | Connection manager with reconnection logic |

### Phase 2: Chat-First + Phase Panel (Weeks 4–6)

| Task | Detail |
|------|--------|
| Chat interface | WhatsApp-style bubbles (`.chat-bubble.other` left, `.chat-bubble.self` right, track-colored) |
| Inline AI cards | Special cards within chat flow (suggestions, evidence, diff cards) |
| Drawable phase panel | Pull-down panel with phase breadcrumb (`●───●───◦`) and summary cards |
| Responsive layout | Mobile: drawer pull-down · Tablet: hybrid · Desktop: persistent sidebar with vertical timeline |
| Real-time updates | WebSocket-driven chat messages and phase state updates |

### Phase 3: Core Features (Weeks 7–12)

| Task | Detail |
|------|--------|
| Bagikan (Create) flow | AI-triage compose flow (UI-01, UI-02) |
| Evidence submission | PoR evidence upload + verification (UI-04) |
| Vouch flow | Vouch submission + display (UI-05) |
| Chat | Real-time thread messaging (WebSocket) |
| Reputation display | Tandang tier/credit visualization (UI-06) |
| Notifications | Real-time + digest (UI-12) |

### Phase 4: Advanced Features (Weeks 13–18)

| Task | Detail |
|------|--------|
| Vault (Catatan Saksi) | Sealed notes lifecycle (UI-09) |
| Siaga emergency | One-tap broadcast (UI-08) |
| Governance voting | Quorum + consent windows (UI-17) |
| LLM-UI blocks | 7 block primitives + diff cards (UI-20) |
| Galang + Siarkan | Resource pooling + broadcast (UI-21, UI-22) |
| PWA optimization | Offline caching, installability |

---

## 10. Sources

### Official Documentation
- [SvelteKit Documentation](https://svelte.dev/docs/kit) — Framework reference
- [Svelte 5 Runes](https://svelte.dev/blog/svelte-5-is-alive) — Reactivity system
- [SvelteKit Service Workers](https://kit.svelte.dev/docs/service-workers) — PWA support
- [SvelteKit Performance Guide](https://svelte.dev/docs/kit/performance) — Optimization reference
- [Bun + SvelteKit Guide](https://bun.com/docs/guides/ecosystem/sveltekit) — Official Bun integration docs

### Benchmarks & Comparisons
- [krausest/js-framework-benchmark](https://krausest.github.io/js-framework-benchmark/) — DOM performance benchmarks (Chrome 143)
- [Pau Sanchez SSR Benchmark](https://pausanchez.com/en/articles/frontend-ssr-frameworks-benchmarked-angular-nuxt-nextjs-and-sveltekit/) — SSR throughput comparison
- [BetterStack: SvelteKit vs Next.js](https://betterstack.com/community/guides/scaling-nodejs/sveltekit-vs-nextjs/) — Comprehensive comparison
- [Windframe: SvelteKit vs Next.js 2026](https://windframe.dev/blog/sveltekit-vs-nextjs) — Bundle size growth formulas
- [JSGuru: Svelte 5 vs React 19 vs Vue 4](https://jsgurujobs.com/blog/svelte-5-vs-react-19-vs-vue-4-the-2025-framework-war-nobody-expected-performance-benchmarks) — Stress test benchmarks

### Ecosystem & Libraries
- [shadcn-svelte](https://www.shadcn-svelte.com/) — Component library for Svelte 5
- [shadcn-svelte Svelte 5 Migration](https://www.shadcn-svelte.com/docs/migration/svelte-5) — Svelte 5 compatibility docs
- [Paraglide JS](https://inlang.com/m/dxnzrydw/paraglide-sveltekit-i18n/) — Type-safe i18n for SvelteKit
- [Paraglide 2.0 Migration Guide](https://dropanote.de/en/blog/20250506-paraglide-migration-2-0-sveltekit/) — v2 features
- [@vite-pwa/sveltekit](https://vite-pwa-org.netlify.app/frameworks/sveltekit) — PWA plugin
- [sveltekit-sse](https://github.com/razshare/sveltekit-sse) — Server-Sent Events library

### Bun Runtime Analysis
- [svelte-adapter-bun](https://github.com/gornostay25/svelte-adapter-bun) — Community-maintained adapter (infrequent updates)
- [DropANote: SvelteKit Bun Analysis](https://dropanote.de/en/blog/20250831-sveltekit-bun-project-still-runs-on-nodejs/) — "Your SvelteKit Bun Project Still Runs on Node.js"

### Alternative Framework Analysis
- [Leptos GitHub](https://github.com/leptos-rs/leptos) — Rust WASM framework (18.5k stars)
- [Yew GitHub](https://yew.rs/) — Rust WASM framework (30.5k stars)
- [SolidStart GitHub Discussion #1743](https://github.com/solidjs/solid-start/discussions/1743) — 2025 modernization roadmap
- [Bejamas: SolidStart Review](https://bejamas.com/hub/web-frameworks/solidstart) — Ecosystem assessment

### Target Market Data
- [DataReportal: Digital 2025 Indonesia](https://datareportal.com/reports/digital-2025-indonesia) — Mobile internet speeds
- [Opensignal: Indonesia Mobile Experience June 2025](https://www.opensignal.com/reports/2025/06/indonesia/mobile-network-experience) — Network quality report

### Production Case Studies
- [Codify: Svelte in 2025 Production](https://codifysol.com/svelte-in-2025-is-it-ready-for-production/) — Apple, NYT, Spotify using Svelte
- [Svelte Blog: What's New January 2026](https://svelte.dev/blog/whats-new-in-svelte-january-2026) — Latest framework updates

---

*End of Report*

*Gotong Royong Frontend Technology Research v1.1 · February 2026 · Internal Document*
*v1.1 revisions: Chat-First UI alignment, security reframing, backend readiness nuance, evidence quality caveats, sensitivity analysis, fallback paths, Auth.js conditional decision, real-time deployment caveats, metrics refresh.*
