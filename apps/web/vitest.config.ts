import { defineConfig, mergeConfig } from 'vitest/config';
import viteConfig from './vite.config';

export default mergeConfig(
	viteConfig,
	defineConfig({
		test: {
			environment: 'node',
			deps: {
				inline: ['svelte-bricks']
			},
			include: ['src/**/*.test.ts'],
			exclude: ['tests/e2e/**']
		}
	})
);
