import { paraglideVitePlugin } from '@inlang/paraglide-js';
import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { SvelteKitPWA } from '@vite-pwa/sveltekit';
import { defineConfig } from 'vite';

const sensitiveRouteDenylist = [
	/^\/masuk(?:[/?]|$)/,
	/^\/beranda(?:[/?]|$)/,
	/^\/terlibat(?:[/?]|$)/,
	/^\/bantu(?:[/?]|$)/,
	/^\/notifikasi(?:[/?]|$)/,
	/^\/profil(?:[/?]|$)/,
	/^\/admin(?:[/?]|$)/
];

const apiProxyTarget = process.env.GR_API_PROXY_TARGET?.trim();

export default defineConfig({
	optimizeDeps: {
		include: ['svelte-bricks']
	},
	ssr: {
		noExternal: ['svelte-bricks']
	},
	server: apiProxyTarget
		? {
				proxy: {
					'/v1': {
						target: apiProxyTarget,
						changeOrigin: true
					}
				}
			}
		: undefined,
	plugins: [
		paraglideVitePlugin({
			project: './project.inlang',
			outdir: './src/lib/paraglide',
			strategy: ['cookie', 'baseLocale']
		}),
		tailwindcss(),
		sveltekit(),
		SvelteKitPWA({
			registerType: 'autoUpdate',
			manifest: {
				name: 'Gotong Royong',
				short_name: 'Gotong',
				description: 'Platform kolaborasi warga untuk bergerak dari masalah ke solusi bersama.',
				theme_color: '#c05621',
				background_color: '#fffbf5',
				display: 'standalone',
				start_url: '/',
				scope: '/',
				lang: 'id',
				icons: [
					{
						src: 'pwa-192x192.png',
						sizes: '192x192',
						type: 'image/png',
						purpose: 'any'
					},
					{
						src: 'pwa-512x512.png',
						sizes: '512x512',
						type: 'image/png',
						purpose: 'any'
					},
					{
						src: 'pwa-512x512-maskable.png',
						sizes: '512x512',
						type: 'image/png',
						purpose: 'maskable'
					}
				]
			},
			workbox: {
				cleanupOutdatedCaches: true,
				// Prevent the SvelteKit wrapper from forcing prerender glob patterns when this app has no prerendered pages.
				modifyURLPrefix: {},
				globPatterns: [
					'client/**/*.{js,css,ico,png,svg,webp,webmanifest,woff2,txt}',
					'client/_app/version.json'
				],
				navigateFallback: null,
				navigateFallbackDenylist: sensitiveRouteDenylist,
				runtimeCaching: []
			}
		})
	]
});
