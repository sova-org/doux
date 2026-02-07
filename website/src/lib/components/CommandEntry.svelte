<script lang="ts">
    import type { Snippet } from "svelte";

    interface Props {
        name: string;
        type?: "number" | "boolean" | "enum" | "source";
        min?: number;
        max?: number;
        default?: number | string | boolean;
        unit?: string;
        values?: string[];
        mod?: boolean;
        children: Snippet;
    }

    let {
        name,
        type,
        min,
        max,
        default: defaultValue,
        unit,
        values,
        mod: modulatable,
        children,
    }: Props = $props();

    let detailsEl: HTMLDetailsElement;

    function formatRange(): string | null {
        if (min !== undefined && max !== undefined) {
            return `${min}–${max}`;
        }
        if (min !== undefined) {
            return `≥${min}`;
        }
        if (max !== undefined) {
            return `≤${max}`;
        }
        return null;
    }

    function onToggle() {
        if (!detailsEl.open) return;
        const section = detailsEl.closest("section.category");
        if (!section) return;
        for (const d of section.querySelectorAll("details")) {
            if (d !== detailsEl) d.open = false;
        }
    }
</script>

<details id={name} bind:this={detailsEl} ontoggle={onToggle}>
    <summary>
        <span class="name">{name}{#if modulatable}<span class="mod" title="supports inline modulation">~</span>{/if}</span>
        {#if type && type !== "source"}
            <span class="meta">
                <span class="type">{type}</span>
                {#if formatRange()}
                    <span class="range"
                        >{formatRange()}{#if unit}
                            {unit}{/if}</span
                    >
                {:else if unit}
                    <span class="unit">{unit}</span>
                {/if}
                {#if defaultValue !== undefined}
                    <span class="default">={defaultValue}</span>
                {/if}
                {#if values}
                    <span class="values">{values.join(" | ")}</span>
                {/if}
            </span>
        {/if}
    </summary>
    <div class="entry-content">
        {@render children()}
    </div>
</details>

<style>
    details {
        border-bottom: 1px solid #ddd;
    }

    summary {
        padding: 8px 0;
        cursor: pointer;
        display: flex;
        align-items: center;
        gap: 12px;
        list-style: none;
    }

    summary::-webkit-details-marker {
        display: none;
    }

    summary::before {
        content: "▶";
        font-size: 0.7em;
        color: #999;
        transition: transform 0.15s;
    }

    details[open] summary::before {
        transform: rotate(90deg);
    }

    .name {
        font-weight: bold;
    }

    .mod {
        display: inline-block;
        font-size: 0.75em;
        vertical-align: super;
        color: #999;
        margin-left: 1px;
    }

    .meta {
        display: inline-flex;
        gap: 6px;
        font-size: 0.85em;
    }

    .meta span {
        padding: 2px 6px;
        background: #f5f5f5;
        color: #666;
    }

    .type {
        color: #666 !important;
    }

    .range {
        color: #666 !important;
    }

    .default {
        color: #999 !important;
    }

    .entry-content {
        padding: 0 0 12px 16px;
    }
</style>
