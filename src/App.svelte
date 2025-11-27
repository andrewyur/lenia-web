<script lang="ts" module>
  type AppContext = {
    app: App | null
    canvas: HTMLCanvasElement | null
  };
  export const [getAppContext, setAppContext] = createContext<AppContext>();
</script>
<script lang="ts">
  import { onMount } from "svelte";
  import init, { App } from "lenia-web";
  import ControlPanel from "./ControlPanel.svelte";
  import { createContext } from 'svelte';
    import app from "./main";

  let canvas: HTMLCanvasElement;
  const context: AppContext = $state({ 
    app: null,
    canvas: null,
  })

  setAppContext(context)
  let playing = $state(true)
  let scale = $state(3)
  let fps = $state(0);
  const times: DOMHighResTimeStamp[] = [];

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
        entry.contentRect.width / scale,
        entry.contentRect.height / scale,
      );
    }, 50);
  });

  onMount(async () => {
    await init();

    context.canvas = canvas
    context.app = await App.new(canvas)

    const animate = () => {
      const now = performance.now();
      while (times.length > 0 && times[0] <= now - 1000) {
        times.shift();
      }
      times.push(now);
      fps = times.length; 

      if(playing) {
        context.app?.step();
      }

      context.app?.render_frame();
      requestAnimationFrame(animate)
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

  <ControlPanel bind:fps bind:playing bind:scale/>
</main>
