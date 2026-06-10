<script lang="ts">
    import { searchIndex } from "$lib/search/index";

    interface Category {
        slug: string;
        title: string;
        group: string;
    }

    interface Props {
        categories: Category[];
    }

    let { categories }: Props = $props();

    let activeSlug = $state("");
    let filter = $state("");

    const haystacks = new Map<string, string>();
    for (const e of searchIndex) {
        const prev = haystacks.get(e.slug) ?? "";
        haystacks.set(e.slug, `${prev} ${[e.name, ...e.aliases].join(" ").toLowerCase()}`);
    }

    function matches(cat: Category): boolean {
        const q = filter.trim().toLowerCase();
        if (!q) return true;
        return (haystacks.get(cat.slug) ?? cat.title.toLowerCase()).includes(q);
    }

    function grouped(group: string): Category[] {
        return categories.filter((c) => c.group === group && matches(c));
    }

    $effect(() => {
        const sections = document.querySelectorAll<HTMLElement>("section.category");
        let ticking = false;

        function update() {
            let current = "";
            for (const section of sections) {
                if (section.getBoundingClientRect().top <= 80) {
                    current = section.id;
                }
            }
            activeSlug = current;
            ticking = false;
        }

        function onScroll() {
            if (!ticking) {
                requestAnimationFrame(update);
                ticking = true;
            }
        }

        window.addEventListener("scroll", onScroll, { passive: true });
        update();

        return () => window.removeEventListener("scroll", onScroll);
    });
</script>

<aside class="sidebar">
    <div class="filter">
        <input bind:value={filter} placeholder="filter" spellcheck="false" />
    </div>
    {#each ["sources", "synthesis", "effects"] as group (group)}
        {@const cats = grouped(group)}
        {#if cats.length}
            <div class="sidebar-section label">{group}</div>
            {#each cats as cat (cat.slug)}
                <a
                    href="#{cat.slug}"
                    class="category-link"
                    class:active={activeSlug === cat.slug}
                >
                    {cat.title}
                </a>
            {/each}
        {/if}
    {/each}
</aside>

<style>
    .filter {
        position: sticky;
        top: 0;
        background: var(--bg);
        padding: 8px 12px;
        border-bottom: 1px solid var(--hairline);
    }

    .filter input {
        border: 1px solid var(--hairline);
        background: var(--bg);
        padding: 4px 8px;
        font-size: 12px;
    }

    .category-link {
        display: block;
        padding: 3px 16px;
        border-left: 2px solid transparent;
        color: var(--muted);
        text-decoration: none;
        font-size: 13px;
    }

    .category-link:hover {
        color: var(--ink);
    }

    .category-link.active {
        color: var(--ink);
        border-left-color: var(--accent);
    }
</style>
