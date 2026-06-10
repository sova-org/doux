/**
 * Rehype plugin: turn inline code spans whose text matches a known doux
 * command name, alias, or section slug into links to the reference anchor.
 * Skips code inside <pre> or <a>.
 */
export default function douxLinks({ names }) {
	return (tree) => walk(tree, false);

	function walk(node, blocked) {
		if (!node.children) return;
		const isBlocked = blocked || node.tagName === 'pre' || node.tagName === 'a';
		node.children = node.children.map((child) => {
			if (
				!isBlocked &&
				child.type === 'element' &&
				child.tagName === 'code' &&
				child.children?.length === 1 &&
				child.children[0].type === 'text' &&
				names.has(child.children[0].value)
			) {
				return {
					type: 'element',
					tagName: 'a',
					properties: {
						className: ['xref'],
						href: `/reference#${names.get(child.children[0].value)}`
					},
					children: [child]
				};
			}
			walk(child, isBlocked);
			return child;
		});
	}
}
