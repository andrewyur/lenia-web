<script lang="ts">
    import { getAppContext } from "../App.svelte";


    let {
        scale = $bindable()
    }: {
        scale: number
    } = $props();

    const context = getAppContext();


    $effect(() => {
        const boundingRect = context.canvas?.getBoundingClientRect();
        if(boundingRect) {
            context.app?.handle_resize(
                boundingRect.width / scale,
                boundingRect.height / scale,
            );
        }
    })
</script>

<div class="rounded-lg bg-base-100 flex items-center flex-row gap-3 justify-center">
    <p class="label italic">Pixel Ratio</p>
    <label class="input w-30">
        <span class=" w-min label">1 to</span>
        <input type="number" bind:value={scale} step="1" min="1"/>
    </label>
</div>