<script lang="ts">
    import type { HTMLButtonAttributes } from "svelte/elements";

    let { path, "class": className, ...rest }: { path: string } & Partial<HTMLButtonAttributes> = $props()
</script>

<button
    class={"btn btn-square! btn-ghost! " + className}
    {...rest}
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
            d={path}
        />
    </svg>
</button>