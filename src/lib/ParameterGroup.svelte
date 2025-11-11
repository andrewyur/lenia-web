<script lang="ts">
    import type { Snippet } from "svelte";

    let { children, title }: { children: Snippet, title: string} = $props()
</script>

<div class="collapse w-80 collapse-arrow">
    <input type="radio" name="control-panel"/>
    <div class="collapse-title">{title}</div>
    <div class="collapse-content flex flex-col gap-7">
        {@render children()}
    </div>
</div>
