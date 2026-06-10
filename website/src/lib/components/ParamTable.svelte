<script lang="ts">
    interface Param {
        name: string;
        alias?: string;
        range?: string;
        default?: string;
        mod?: boolean;
        description: string;
    }

    let { params }: { params: Param[] } = $props();
</script>

<table>
    <thead>
        <tr>
            <th>param</th>
            <th>alias</th>
            <th>range</th>
            <th>default</th>
            <th>description</th>
        </tr>
    </thead>
    <tbody>
        {#each params as p}
            <tr>
                <td class="name"
                    >{p.name}{#if p.mod}<span
                            class="mod"
                            title="supports inline modulation">~</span
                        >{/if}</td
                >
                <td class="alias">{p.alias ?? "—"}</td>
                <td class="range">{p.range ?? "—"}</td>
                <td class="default">{p.default ?? "—"}</td>
                <td class="description">{p.description}</td>
            </tr>
        {/each}
    </tbody>
</table>

<style>
    table {
        width: 100%;
        border-collapse: collapse;
        font-family: var(--font-mono);
        font-size: 13px;
        font-variant-numeric: tabular-nums;
        margin: 14px 0;
        border-top: 1px solid var(--hairline);
    }

    th {
        text-align: left;
        font-weight: 400;
        font-size: 11px;
        text-transform: uppercase;
        letter-spacing: 0.08em;
        color: var(--faint);
        padding: 6px 10px 6px 0;
        border-bottom: 1px solid var(--hairline);
    }

    td {
        padding: 7px 10px 7px 0;
        border-bottom: 1px solid var(--hairline);
        vertical-align: top;
    }

    .name {
        font-weight: 600;
        white-space: nowrap;
    }

    .mod {
        display: inline-block;
        font-size: 0.75em;
        vertical-align: super;
        color: var(--accent);
        margin-left: 1px;
    }

    .alias {
        color: var(--muted);
        white-space: nowrap;
    }

    .range,
    .default {
        color: var(--muted);
        white-space: nowrap;
    }

    .description {
        font-family: var(--font-sans);
        font-size: 13px;
        color: var(--ink-soft);
    }
</style>
