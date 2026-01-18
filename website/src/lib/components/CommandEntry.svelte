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
        children,
    }: Props = $props();

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
</script>

<details id={name}>
    <summary>
        <span class="name">{name}</span>
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
    <div class="content">
        {@render children()}
    </div>
</details>

<style>
    details {
        margin: 16px 0;
        background: #f5f5f5;
        border: 1px solid #ccc;
    }

    summary {
        padding: 10px 14px;
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

    .meta {
        display: inline-flex;
        gap: 6px;
        font-size: 0.85em;
    }

    .meta span {
        padding: 2px 6px;
        background: #eee;
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

    .content {
        padding: 0 14px 14px;
        border-top: 1px solid #ddd;
    }
</style>
