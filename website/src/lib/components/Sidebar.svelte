<script lang="ts">
    interface NavItem {
        name: string;
        category: string;
        group: string;
    }

    interface Props {
        items: NavItem[];
    }

    let { items }: Props = $props();

    let expanded = $state<Record<string, boolean>>({});

    function toggleCategory(category: string) {
        expanded[category] = !expanded[category];
    }

    function capitalize(str: string): string {
        return str.charAt(0).toUpperCase() + str.slice(1);
    }

    const categoryNames: Record<string, string> = {
        plaits: "Complex",
        io: "Audio Input",
        am: "Amplitude Modulation",
        rm: "Ring Modulation",
        lowpass: "Lowpass Filter",
        highpass: "Highpass Filter",
        bandpass: "Bandpass Filter",
        ftype: "Filter Type",
    };

    function formatCategory(str: string): string {
        return categoryNames[str] ?? capitalize(str);
    }

    const grouped = $derived.by(() => {
        const groups: Record<string, Record<string, NavItem[]>> = {
            sources: {},
            synthesis: {},
            effects: {},
        };

        for (const item of items) {
            const group = item.group;
            const category = item.category;
            if (!groups[group]) continue;
            if (!groups[group][category]) {
                groups[group][category] = [];
            }
            groups[group][category].push(item);
        }

        return groups;
    });
</script>

<aside class="sidebar">
    {#each ["sources", "synthesis", "effects"] as group (group)}
        <div class="sidebar-section">{capitalize(group)}</div>
        {#each Object.entries(grouped[group]) as [category, navItems] (category)}
            <button
                class="category-toggle"
                onclick={() => toggleCategory(category)}
            >
                {formatCategory(category)}
            </button>
            {#if expanded[category]}
                <div class="commands">
                    {#each navItems as item (item.name)}
                        <a href="#{item.name}" class="command-link"
                            >{item.name}</a
                        >
                    {/each}
                </div>
            {/if}
        {/each}
    {/each}
</aside>

<style>
    .sidebar-section {
        margin-top: 16px;
    }

    .category-toggle {
        display: block;
        width: 100%;
        background: none;
        border: none;
        color: #666;
        cursor: pointer;
        padding: 4px 8px;
        font-size: inherit;
        font-family: inherit;
        text-align: left;
    }

    .category-toggle:hover {
        color: #000;
        background: #f5f5f5;
    }

    .commands {
        padding-left: 18px;
    }

    .command-link {
        display: block;
        color: #666;
        text-decoration: none;
        padding: 2px 8px;
        font-size: 0.9em;
    }

    .command-link:hover {
        color: #000;
        background: #f5f5f5;
    }
</style>
