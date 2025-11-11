<script lang="ts">
    let {
        name,
        min,
        max,
        step,
        value = $bindable(),
        registerRandomize,
    }: {
        name: string;
        min: number;
        max: number;
        step: number;
        value: number;
        registerRandomize?: (fn: () => void) => void
    } = $props();

    let rawValue = $state(value);

    let debounceTimeout: null | number = null;

    const handleValue = () => {
        if (debounceTimeout) clearTimeout(debounceTimeout);
        debounceTimeout = setTimeout(() => {
            value = rawValue;
        }, 100);
    };

    registerRandomize?.(() => {
        rawValue = (Math.round(Math.random() * (max - min) * (1/step)) * step);
        value = rawValue
    })
</script>

<!-- svelte-ignore a11y_label_has_associated_control -->
<div class="rounded-lg bg-base-100 flex items-center flex-col gap-3">
    <div class="flex flex-row align-middle justify-around gap-4">
        <p class="label italic">{name}</p>
        <input
            type="number"
            class="input w-25"
            {step}
            bind:value={rawValue}
            oninput={handleValue}
        />
    </div>
    <input
    class="w-full"
        type="range"
        {min}
        {max}
        bind:value={rawValue}
        {step}
        oninput={handleValue}
    />
</div>
