<script lang="ts">
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

    function grouped(group: string): Category[] {
        return categories.filter((c) => c.group === group);
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
    {#each ["sources", "synthesis", "effects"] as group (group)}
        <div class="sidebar-section">{group}</div>
        {#each grouped(group) as cat (cat.slug)}
            <a
                href="#{cat.slug}"
                class="category-link"
                class:active={activeSlug === cat.slug}
            >
                {cat.title}
            </a>
        {/each}
    {/each}
</aside>

<style>
    .category-link {
        display: block;
        padding: 4px 16px;
        color: #666;
        text-decoration: none;
    }

    .category-link:hover {
        color: #000;
        background: #f5f5f5;
    }

    .category-link.active {
        color: #000;
        background: #f5f5f5;
    }
</style>
