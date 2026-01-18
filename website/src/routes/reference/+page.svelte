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

    interface NavItem {
        name: string;
        category: string;
        group: string;
    }

    interface Props {
        data: {
            categories: Category[];
            navigation: NavItem[];
        };
    }

    let { data }: Props = $props();
</script>

<Sidebar items={data.navigation} />

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
    .category-title {
        border-bottom: 1px solid #ccc;
        padding-bottom: 8px;
        margin-bottom: 24px;
        font-size: 1.2em;
    }

    .category :global(h2:not(.category-title)) {
        background: #f5f5f5;
        border: 1px solid #ccc;
        font-size: 1em;
        font-weight: normal;
        margin: 24px 0 8px;
        padding: 8px 12px;
    }
</style>
