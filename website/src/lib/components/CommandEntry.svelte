<script lang="ts">
    import type { Snippet } from "svelte";

    interface Props {
        name: string;
        aliases?: string;
        type?: "number" | "boolean" | "enum" | "source" | "string";
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
        aliases,
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
        <span class="name">{name}</span>
        {#if modulatable}
            <span
                class="mod"
                title="accepts inline modulation: 0~1:2 (cycle) · a>b:t (transition) · >b:t (slew) · min^max (envelope) · min?max:t (random)"
                >~mod</span
            >
        {/if}
        {#if aliases}
            <span class="aliases">{aliases}</span>
        {/if}
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
        border-bottom: 1px solid var(--hairline);
    }

    summary {
        padding: 7px 0;
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
        font-size: 0.6em;
        color: var(--faint);
        transition: transform 0.15s;
    }

    details[open] summary::before {
        transform: rotate(90deg);
    }

    .name {
        font-family: var(--font-mono);
        font-size: 13px;
        font-weight: 600;
    }

    .mod {
        font-family: var(--font-mono);
        font-size: 10px;
        color: var(--accent);
        border: 1px solid color-mix(in srgb, var(--accent) 40%, transparent);
        padding: 0 5px;
        white-space: nowrap;
    }

    .aliases {
        font-family: var(--font-mono);
        font-size: 12px;
        color: var(--faint);
    }

    .meta {
        display: inline-flex;
        gap: 6px;
        font-family: var(--font-mono);
        font-size: 11px;
    }

    .meta span {
        padding: 1px 6px;
        border: 1px solid var(--hairline);
        color: var(--muted);
    }

    .default {
        color: var(--faint) !important;
    }

    .entry-content {
        padding: 0 0 12px 16px;
    }
</style>
