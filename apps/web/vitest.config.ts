import { defineConfig, mergeConfig } from 'vitest/config';
import viteConfig from './vite.config';

export default mergeConfig(
	viteConfig,
	defineConfig({
		test: {
			environment: 'node',
			include: ['src/**/*.test.ts'],
			exclude: ['tests/e2e/**']
		}
	})
);
