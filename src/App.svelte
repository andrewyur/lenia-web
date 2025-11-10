<script lang="ts" module>
  export const [getAppContext, setAppContext] = createContext<{ app: App | null }>();
</script>
<script lang="ts">
  import { onMount } from "svelte";
  import init, { App } from "lenia-web";
  import ControlPanel from "./ControlPanel.svelte";
  import { createContext } from 'svelte';

  let canvas: HTMLCanvasElement;
  const context = $state({ app: null as App | null })
  setAppContext(context)
  let playing = $state(true)

  let clickEvent: null | MouseEvent = $state(null);
  let resizeTimeout: null | number = $state(null);


  function randomize() {
    if (!clickEvent) return;

    const rect = canvas.getBoundingClientRect();

    const x = clickEvent.clientX - rect.left;
    const y = clickEvent.clientY - rect.top;

    context.app?.randomize(
      x * (canvas.width / rect.width),
      y * (canvas.height / rect.height),
    );

    requestAnimationFrame(randomize);
  }

  function handleMousedown(e: MouseEvent) {
    clickEvent = e;
    randomize();
  }
  function handleMousemove(e: MouseEvent) {
    if (clickEvent) clickEvent = e;
  }
  function handleMouseup() {
    clickEvent = null;
  }

  let resizeObserver = new ResizeObserver(([entry]) => {
    if (resizeTimeout) clearTimeout(resizeTimeout);
    resizeTimeout = setTimeout(() => {
      context.app?.handle_resize(
        entry.contentRect.width / 3,
        entry.contentRect.height / 3,
      );
    }, 50);
  });

  onMount(async () => {
    await init();

    context.app = await App.new(canvas)

    const animate = () => {
      context.app?.render_frame(playing);
      requestAnimationFrame(animate);
    };
    animate();

    resizeObserver.observe(canvas);
  });
</script>

<main class="w-full h-full">
  <canvas
    bind:this={canvas}
    onmousedown={handleMousedown}
    onmousemove={handleMousemove}
    onmouseup={handleMouseup}
    class="w-full h-full"
    style="image-rendering: pixelated;"
  ></canvas>

  <ControlPanel bind:playing />
</main>
