<script lang="ts">
    import type { ComputeConfig, RandomConfig } from "lenia-web";
    import Parameter from "./lib/Parameter.svelte";
    import { getAppContext } from "./App.svelte";
    import ScaleTuner from "./lib/ScaleTuner.svelte";
    import SvgButton from "./lib/SvgButton.svelte";
    import ParameterGroup from "./lib/ParameterGroup.svelte";

    let {
        playing = $bindable(true),
        scale = $bindable(),
        fps = $bindable()
    }: {
        playing: boolean;
        scale: number;
        fps: number;
    } = $props();

    const randomizeFnStore = new Set<() => void>();
    const registerRandomize = (fn: () => void) => {
        randomizeFnStore.add(fn);
    };
    const randomize = () => {
        randomizeFnStore.forEach((fn) => fn());
    };

    const context = getAppContext();

    let randomValues: RandomConfig = $state({
        x: 0,
        y: 0,
        seed: Math.round(Math.random() * 10000),
        density: 0.5,
        use_brush: 1,
        size: 10,
    });

    let computeValues: ComputeConfig = $state({
        time_step: 50,
        m: 0.135,
        s: 0.015,
    });

    $effect(() => {
        if (randomValues) context.app?.set_random_values(randomValues);
    });
    $effect(() => {
        if (computeValues) context.app?.set_compute_values(computeValues);
    });

    let visible = $state(true);
</script>

<SvgButton
    class="absolute bottom-10 right-10"
    onclick={() => (visible = true)}
    hidden={visible}
    aria-label="show control panel"
    path="M10.5 6h9.75M10.5 6a1.5 1.5 0 1 1-3 0m3 0a1.5 1.5 0 1 0-3 0M3.75 6H7.5m3 12h9.75m-9.75 0a1.5 1.5 0 0 1-3 0m3 0a1.5 1.5 0 0 0-3 0m-3.75 0H7.5m9-6h3.75m-3.75 0a1.5 1.5 0 0 1-3 0m3 0a1.5 1.5 0 0 0-3 0m-9.75 0h9.75"
/>

<div
    class="absolute right-5 bottom-5 flex flex-col bg-base-100 p-4 rounded-lg"
    hidden={!visible}
>
    <div class="flex flex-row gap-5">
        <SvgButton
            onclick={() => (playing = !playing)}
            aria-label="stop/start the game"
            path={playing
                ? "M15.75 5.25v13.5m-7.5-13.5v13.5"
                : "M5.25 5.653c0-.856.917-1.398 1.667-.986l11.54 6.347a1.125 1.125 0 0 1 0 1.972l-11.54 6.347a1.125 1.125 0 0 1-1.667-.986V5.653Z"}
        />
        <SvgButton
            onclick={() => context.app?.clear()}
            aria-label="clear the canvas"
            path="M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0 3.181 3.183a8.25 8.25 0 0 0 13.803-3.7M4.031 9.865a8.25 8.25 0 0 1 13.803-3.7l3.181 3.182m0-4.991v4.99"
        />

        <div class="grow"></div>
        <SvgButton
            onclick={() => (visible = false)}
            aria-label="hide the control panel"
            path="M6 18 18 6M6 6l12 12"
        />
    </div>
    
    <ParameterGroup title="Screen Parameters">
        <ScaleTuner bind:scale />
    </ParameterGroup>

    <ParameterGroup title="Simulation Parameters">
        <button
            class="btn"
            onclick={randomize}
            aria-label="randomize parameters"
        >
        Randomize Parameters
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
                    d="m18 14l4 4l-4 4m0-20l4 4l-4 4 M2 18h1.973a4 4 0 0 0 3.3-1.7l5.454-8.6a4 4 0 0 1 3.3-1.7H22M2 6h1.972a4 4 0 0 1 3.6 2.2M22 18h-6.041a4 4 0 0 1-3.3-1.8l-.359-.45"
                />
    </svg>
    </button>
        <Parameter
            {registerRandomize}
            name="Time Step"
            min={1}
            max={50}
            bind:value={computeValues.time_step}
            step={1}
        />
        <Parameter
            {registerRandomize}
            name="m"
            min={0}
            max={1}
            bind:value={computeValues.m}
            step={0.0001}
        />
        <Parameter
            {registerRandomize}
            name="s"
            min={0}
            max={1}
            bind:value={computeValues.s}
            step={0.0001}
        />
    </ParameterGroup>

    <ParameterGroup title="Randomizer Brush Parameters">
        <Parameter
            name="Brush Size"
            min={1}
            max={20}
            bind:value={randomValues.size}
            step={1}
        />
        <Parameter
            name="Density"
            min={0}
            max={1}
            bind:value={randomValues.density}
            step={0.01}
        />
    </ParameterGroup>
</div>

<div class="absolute top-0 left-0">
    {fps} F/s
</div>