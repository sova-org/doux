<script lang="ts">
    import type { Component } from "svelte";
    import Sidebar from "$lib/components/Sidebar.svelte";

    interface Category {
        path: string;
        title: string;
        slug: string;
        group: string;
        order: number;
        component: Component;
    }

    interface Props {
        data: {
            categories: Category[];
        };
    }

    let { data }: Props = $props();

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

<main class="content">
    {#each data.categories as category}
        {@const Component = category.component}
        <section id={category.slug} class="category">
            <h2 class="category-title">{category.title}</h2>
            <Component />
        </section>
    {/each}
</main>

<style>
    .category {
        display: grid;
        grid-template-columns: repeat(2, 1fr);
        column-gap: 32px;
    }

    .category-title {
        grid-column: 1 / -1;
        border-bottom: 1px solid #ccc;
        padding-bottom: 8px;
        margin-top: 2em;
        margin-bottom: 16px;
        font-size: 1.2em;
    }

    .category:first-of-type .category-title {
        margin-top: 0;
    }

    .category :global(p),
    .category :global(h2:not(.category-title)),
    .category :global(ul),
    .category :global(ol) {
        grid-column: 1 / -1;
    }

    .category :global(h2:not(.category-title)) {
        font-size: 1em;
        font-weight: normal;
        margin: 2em 0 8px;
        padding: 6px 0;
        border-bottom: 1px solid #ddd;
        color: #666;
    }

    @media (max-width: 768px) {
        .category {
            grid-template-columns: 1fr;
        }
    }
</style>
