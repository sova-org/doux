import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';
import { mdsvex } from 'mdsvex';
import { readFileSync, readdirSync } from 'node:fs';
import douxLinks from './rehype-doux-links.js';

const pkg = JSON.parse(readFileSync('package.json', 'utf-8'));

// Known names for `code`-span auto-linking (mirrors src/lib/search/index.ts,
// duplicated knowingly: this runs in node at config time, not in the app).
// Dev server restart needed when content names change.
function contentNames() {
	const names = new Map();
	for (const file of readdirSync('src/content')) {
		if (!file.endsWith('.md')) continue;
		const source = readFileSync(`src/content/${file}`, 'utf-8');
		const slug = source.match(/^slug:\s*"?([^"'\n]+)"?\s*$/m)?.[1];
		if (slug) names.set(slug, slug);
		for (const m of source.matchAll(/<CommandEntry\s+([^>]*?)>/g)) {
			const name = m[1].match(/name="([^"]+)"/)?.[1];
			if (!name) continue;
			names.set(name, name);
			const aliases = m[1].match(/aliases="([^"]+)"/)?.[1];
			if (aliases) {
				for (const alias of aliases.split(',')) names.set(alias.trim(), name);
			}
		}
	}
	return names;
}

/** @type {import('@sveltejs/kit').Config} */
const config = {
	extensions: ['.svelte', '.md'],
	preprocess: [
		vitePreprocess(),
		mdsvex({
			extensions: ['.md'],
			rehypePlugins: [[douxLinks, { names: contentNames() }]]
		})
	],
	kit: {
		adapter: adapter({
			pages: 'build',
			assets: 'build',
			fallback: '404.html'
		}),
		appDir: 'app',
		version: { name: pkg.version }
	}
};

export default config;
