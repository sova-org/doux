<script lang="ts">
    import type { Component } from "svelte";
    import Sidebar from "$lib/components/Sidebar.svelte";
    import HoverCard from "$lib/components/HoverCard.svelte";

    interface Category {
        path: string;
        title: string;
        slug: string;
        group: string;
        order: number;
        related?: string[];
        component: Component;
    }

    interface Props {
        data: {
            categories: Category[];
        };
    }

    let { data }: Props = $props();

    function relatedOf(category: Category): Category[] {
        return (category.related ?? [])
            .map((slug) => data.categories.find((c) => c.slug === slug))
            .filter((c) => c !== undefined);
    }

    function openHashTarget() {
        const hash = location.hash.slice(1);
        if (!hash) return;
        const el = document.getElementById(hash);
        if (el instanceof HTMLDetailsElement) {
            el.open = true;
        }
    }

    $effect(() => {
        openHashTarget();
        window.addEventListener("hashchange", openHashTarget);
        return () => window.removeEventListener("hashchange", openHashTarget);
    });
</script>

<Sidebar categories={data.categories} />
<HoverCard />

<main class="content">
    <p class="mod-legend">
        <span class="mod-badge">~mod</span> marks parameters that accept
        <a href="/#modulation">inline modulation</a> — e.g.
        <code>lpf/200~4000:2</code>, <code>verb/0>0.9:8</code>,
        <code>combfreq/100?1200:0.5</code>.
    </p>
    {#each data.categories as category}
        {@const Component = category.component}
        {@const related = relatedOf(category)}
        <section id={category.slug} class="category">
            <header class="category-header">
                <span class="label">{category.group}</span>
                <h2 class="category-title">{category.title}</h2>
                {#if related.length}
                    <span class="see-also">
                        <span class="label">see also</span>
                        {#each related as rel (rel.slug)}
                            <a href="#{rel.slug}" class="see-also-link">{rel.title}</a>
                        {/each}
                    </span>
                {/if}
            </header>
            <Component />
        </section>
    {/each}
</main>

<style>
    .mod-legend {
        margin: 1em 0 0;
        font-size: 13px;
        color: var(--muted);
    }

    .mod-badge {
        font-family: var(--font-mono);
        font-size: 10px;
        color: var(--accent);
        border: 1px solid color-mix(in srgb, var(--accent) 40%, transparent);
        padding: 0 5px;
        white-space: nowrap;
    }

    .category-header {
        display: flex;
        align-items: baseline;
        gap: 12px;
        border-bottom: 1px solid var(--hairline-strong);
        padding-bottom: 6px;
        margin-top: 3em;
        margin-bottom: 12px;
    }

    .category:first-of-type .category-header {
        margin-top: 1em;
    }

    .category-title {
        margin: 0;
        font-size: 15px;
    }

    .see-also {
        display: inline-flex;
        align-items: baseline;
        gap: 8px;
        margin-left: auto;
    }

    .see-also-link {
        font-family: var(--font-mono);
        font-size: 12px;
        color: var(--muted);
        text-decoration: none;
        border: 1px solid var(--hairline);
        padding: 1px 6px;
    }

    .see-also-link:hover {
        color: var(--accent);
        border-color: var(--accent);
    }

    .category :global(h2:not(.category-title)) {
        font-size: 13px;
        font-family: var(--font-mono);
        font-weight: 600;
        margin: 2em 0 8px;
        padding: 6px 0;
        border-bottom: 1px solid var(--hairline);
        color: var(--muted);
    }
</style>
