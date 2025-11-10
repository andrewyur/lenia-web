<script lang="ts">
    import type { ComputeConfig, RandomConfig } from "lenia-web";
    import Parameter from "./lib/Parameter.svelte";
    import { getAppContext } from "./App.svelte";

    let {
        playing = $bindable(true),
    }: {
        playing: boolean;
    } = $props();

    const context = getAppContext();

    let randomValues: RandomConfig = $state({
        x: 0,
        y: 0,
        seed: Math.round(Math.random() * 10000),
        density: 0.5,
        use_brush: 1,
        size: 10
    });

    let computeValues: ComputeConfig = $state({
        time_step: 50,
        m: 0.135,
        s: 0.015,
    });

    $effect(() => {
        if(randomValues) context.app?.set_random_values(randomValues)
    })
    $effect(() => {
        if(computeValues) context.app?.set_compute_values(computeValues)
    })

    let visible = $state(true);
</script>

<button
    class="btn btn-square! btn-ghost! absolute bottom-10 right-10"
    onclick={() => (visible = true)}
    hidden={visible}
    aria-label="show control panel"
>
    <svg
        xmlns="http://www.w3.org/2000/svg"
        fill="none"
        viewBox="0 0 24 24"
        stroke-width="1.5"
        stroke="currentColor"
        class="size-6"
    >
        <path
            stroke-linecap="round"
            stroke-linejoin="round"
            d="M10.5 6h9.75M10.5 6a1.5 1.5 0 1 1-3 0m3 0a1.5 1.5 0 1 0-3 0M3.75 6H7.5m3 12h9.75m-9.75 0a1.5 1.5 0 0 1-3 0m3 0a1.5 1.5 0 0 0-3 0m-3.75 0H7.5m9-6h3.75m-3.75 0a1.5 1.5 0 0 1-3 0m3 0a1.5 1.5 0 0 0-3 0m-9.75 0h9.75"
        />
    </svg>
</button>

<div class="absolute right-5 bottom-5 flex flex-col bg-base-100 p-4 rounded-lg" hidden={!visible}>
    <div class="flex flex-row gap-5">
        <button
            class="btn btn-square! btn-ghost!"
            onclick={() => (playing = !playing)}
            aria-label="stop/start the game"
        >
            <svg
                xmlns="http://www.w3.org/2000/svg"
                fill="none"
                viewBox="0 0 24 24"
                stroke-width="1.5"
                stroke="currentColor"
                class="size-6"
            >
                {#if playing}
                    <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        d="M15.75 5.25v13.5m-7.5-13.5v13.5"
                    />
                {:else}
                    <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        d="M5.25 5.653c0-.856.917-1.398 1.667-.986l11.54 6.347a1.125 1.125 0 0 1 0 1.972l-11.54 6.347a1.125 1.125 0 0 1-1.667-.986V5.653Z"
                    />
                {/if}
            </svg>
        </button>
        <button
            class="btn btn-square! btn-ghost!"
            onclick={() => context.app?.clear()}
            aria-label="clear the canvas"
        >
            <svg
                xmlns="http://www.w3.org/2000/svg"
                fill="none"
                viewBox="0 0 24 24"
                stroke-width="1.5"
                stroke="currentColor"
                class="size-6"
            >
                <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    d="M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0 3.181 3.183a8.25 8.25 0 0 0 13.803-3.7M4.031 9.865a8.25 8.25 0 0 1 13.803-3.7l3.181 3.182m0-4.991v4.99"
                />
            </svg>
        </button>
        <div class="grow"></div>
        <button
            class="btn btn-square! btn-ghost! "
            onclick={() => (visible = false)}
            aria-label="hide the control panel"
        >
            <svg
                xmlns="http://www.w3.org/2000/svg"
                fill="none"
                viewBox="0 0 24 24"
                stroke-width="1.5"
                stroke="currentColor"
                class="size-6"
            >
                <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    d="M6 18 18 6M6 6l12 12"
                />
            </svg>
        </button>
    </div>

    <div class="collapse w-80 collapse-arrow ">
        <input type="checkbox" />
        <div class="collapse-title rounded-lg">Randomizer Brush Parameters</div>
        <div class="collapse-content flex flex-col gap-3">
            <Parameter name="Brush Size" min={1} max={20} bind:value={randomValues.size} step={1}/>
            <Parameter name="Density" min={0} max={1} bind:value={randomValues.density} step={0.01}/>
        </div>
    </div>

    <div class="collapse w-80 collapse-arrow ">
        <input type="checkbox" />
        <div class="collapse-title rounded-lg">Simulation Parameters</div>
        <div class="collapse-content flex flex-col gap-3">
            <Parameter name="Time Step" min={1} max={100} bind:value={computeValues.time_step} step={1}/>
            <Parameter name="m" min={0} max={1} bind:value={computeValues.m} step={0.0001}/>
            <Parameter name="s" min={0} max={1} bind:value={computeValues.s} step={0.0001}/>
        </div>
    </div>
</div>
