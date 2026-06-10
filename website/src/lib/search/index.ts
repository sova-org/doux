import { dev } from '$app/environment';

export interface SearchEntry {
	kind: 'section' | 'command' | 'param';
	name: string;
	aliases: string[];
	description: string;
	slug: string;
	sectionTitle: string;
	group: string;
	anchor: string;
	/** Commands only: formatted type/range/default line, e.g. "number · 0–1 Hz · =0.5". */
	meta?: string;
	mod?: boolean;
}

const rawModules = import.meta.glob('/src/content/*.md', {
	query: '?raw',
	import: 'default',
	eager: true
}) as Record<string, string>;

function frontmatterField(fm: string, field: string): string {
	const m = fm.match(new RegExp(`^${field}:\\s*"?([^"\\n]+)"?\\s*$`, 'm'));
	return m ? m[1].trim() : '';
}

/** First prose line after `from`: skips blank lines and component/script markup. */
function firstProseLine(source: string, from: number): string {
	const lines = source.slice(from).split('\n');
	for (const line of lines.slice(1)) {
		const t = line.trim();
		if (!t) continue;
		if (t.startsWith('<') || t.startsWith('---')) {
			if (t.startsWith('</')) continue;
			return '';
		}
		return t;
	}
	return '';
}

function commandMeta(attrs: string): string {
	const type = attrs.match(/type="([^"]+)"/)?.[1];
	const min = attrs.match(/min=\{([^}]+)\}/)?.[1];
	const max = attrs.match(/max=\{([^}]+)\}/)?.[1];
	const def = attrs.match(/default=\{([^}]+)\}/)?.[1] ?? attrs.match(/default="([^"]+)"/)?.[1];
	const unit = attrs.match(/unit="([^"]+)"/)?.[1];
	const values = attrs
		.match(/values=\{\[([^\]]+)\]\}/)?.[1]
		?.replaceAll('"', '')
		.replaceAll(',', ' |');
	const parts: string[] = [];
	if (type && type !== 'source') parts.push(type);
	if (min !== undefined && max !== undefined) parts.push(`${min}–${max}${unit ? ` ${unit}` : ''}`);
	else if (min !== undefined) parts.push(`≥${min}${unit ? ` ${unit}` : ''}`);
	else if (max !== undefined) parts.push(`≤${max}${unit ? ` ${unit}` : ''}`);
	else if (unit) parts.push(unit);
	if (def !== undefined) parts.push(`=${def}`);
	if (values) parts.push(values);
	return parts.join(' · ');
}

function parseFile(path: string, source: string): SearchEntry[] {
	const entries: SearchEntry[] = [];
	const fmMatch = source.match(/^---\n([\s\S]*?)\n---/);
	if (!fmMatch) {
		if (dev) console.warn(`search index: no frontmatter in ${path}`);
		return entries;
	}
	const fm = fmMatch[1];
	const title = frontmatterField(fm, 'title');
	const slug = frontmatterField(fm, 'slug');
	const group = frontmatterField(fm, 'group');

	const afterScript = source.indexOf('</script>');
	entries.push({
		kind: 'section',
		name: title,
		aliases: [slug],
		description: firstProseLine(source, afterScript === -1 ? fmMatch[0].length : afterScript),
		slug,
		sectionTitle: title,
		group,
		anchor: slug
	});

	const commands: { index: number; name: string }[] = [];
	for (const m of source.matchAll(/<CommandEntry\s+([^>]*?)>/g)) {
		const attrs = m[1];
		const name = attrs.match(/name="([^"]+)"/)?.[1];
		if (!name) {
			if (dev) console.warn(`search index: CommandEntry without name in ${path}`);
			continue;
		}
		const aliases = (attrs.match(/aliases="([^"]+)"/)?.[1] ?? '')
			.split(',')
			.map((a) => a.trim())
			.filter(Boolean);
		commands.push({ index: m.index, name });
		entries.push({
			kind: 'command',
			name,
			aliases,
			description: firstProseLine(source, m.index),
			slug,
			sectionTitle: title,
			group,
			anchor: name,
			meta: commandMeta(attrs),
			mod: /(^|\s)mod(\s|$)/.test(attrs)
		});
	}

	for (const m of source.matchAll(/<ParamTable\s+params=\{\[([\s\S]*?)\]\}/g)) {
		const parent = commands.filter((c) => c.index < m.index).at(-1);
		for (const p of m[1].matchAll(/\{[^}]*\}/g)) {
			const name = p[0].match(/name:\s*"([^"]+)"/)?.[1];
			if (!name) continue;
			const aliases = (p[0].match(/alias:\s*"([^"]+)"/)?.[1] ?? '')
				.split(',')
				.map((a) => a.trim())
				.filter(Boolean);
			entries.push({
				kind: 'param',
				name,
				aliases,
				description: p[0].match(/description:\s*"([^"]+)"/)?.[1] ?? '',
				slug,
				sectionTitle: parent ? `${title} · ${parent.name}` : title,
				group,
				anchor: parent?.name ?? slug
			});
		}
	}

	return entries;
}

export const searchIndex: SearchEntry[] = Object.entries(rawModules).flatMap(([path, source]) =>
	parseFile(path, source)
);
