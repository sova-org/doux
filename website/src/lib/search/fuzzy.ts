import type { SearchEntry } from './index';

export interface SearchResult {
	entry: SearchEntry;
	score: number;
	/** Indices of matched characters in entry.name, for highlighting. */
	nameMatches: number[];
}

/** Subsequence match: returns matched indices or null. Bonus for word-boundary hits. */
function subsequence(query: string, candidate: string): { score: number; indices: number[] } | null {
	const indices: number[] = [];
	let score = 0;
	let ci = 0;
	for (const qc of query) {
		let found = -1;
		while (ci < candidate.length) {
			if (candidate[ci] === qc) {
				found = ci;
				break;
			}
			ci++;
		}
		if (found === -1) return null;
		const prev = indices.at(-1);
		if (found === 0 || candidate[found - 1] === ' ' || candidate[found - 1] === '-') {
			score += 8; // boundary-anchored
		} else if (prev !== undefined && found === prev + 1) {
			score += 6; // contiguous run
		} else {
			score += 1; // scattered, penalize gap
			score -= Math.min(3, found - (prev ?? -1) - 1) * 0.5;
		}
		indices.push(found);
		ci = found + 1;
	}
	return { score, indices };
}

function fieldScore(query: string, field: string): { score: number; indices: number[] } | null {
	if (!field) return null;
	const f = field.toLowerCase();
	if (f === query) return { score: 100, indices: [...query].map((_, i) => i) };
	if (f.startsWith(query)) return { score: 60 + 10 / f.length, indices: [...query].map((_, i) => i) };
	const idx = f.indexOf(query);
	if (idx !== -1) return { score: 40, indices: [...query].map((_, i) => idx + i) };
	return subsequence(query, f);
}

export function search(query: string, index: SearchEntry[], limit = 20): SearchResult[] {
	const q = query.trim().toLowerCase();
	if (!q) return [];
	const results: SearchResult[] = [];
	for (const entry of index) {
		let best = 0;
		let nameMatches: number[] = [];
		const name = fieldScore(q, entry.name);
		if (name) {
			best = name.score * 3;
			nameMatches = name.indices;
		}
		for (const alias of entry.aliases) {
			const s = fieldScore(q, alias);
			if (s && s.score * 2.5 > best) best = s.score * 2.5;
		}
		const section = fieldScore(q, entry.sectionTitle);
		if (section && section.score * 1.5 > best) best = section.score * 1.5;
		// descriptions: substring only — subsequence over long text is all noise
		if (entry.description.toLowerCase().includes(q) && 35 > best) best = 35;
		// commands are what users type; break section/command ties in their favor
		if (entry.kind === 'command') best *= 1.05;
		if (best > 8) results.push({ entry, score: best, nameMatches });
	}
	results.sort((a, b) => b.score - a.score);
	return results.slice(0, limit);
}
