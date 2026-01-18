import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';
import { mdsvex } from 'mdsvex';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	extensions: ['.svelte', '.md'],
	preprocess: [
		vitePreprocess(),
		mdsvex({ extensions: ['.md'] })
	],
	kit: {
		adapter: adapter({
			pages: 'build',
			assets: 'build',
			fallback: '404.html'
		}),
		appDir: 'app'
	}
};

export default config;
