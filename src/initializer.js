export default function myInitializer () {
  return {
    onSuccess: async (wasm) => {
      const canvas = document.getElementById("canvas");

      const playPauseButton = document.getElementById("play-pause");
      let playing = true;

      playPauseButton.addEventListener("click", () => {
        playing = !playing;
      })

      const app = await wasm.app_new(canvas)

      function animate() {
        app.render_frame(playing)
        requestAnimationFrame(animate)
      }

      let timeout = null;

      let resizeObserver = new ResizeObserver(([ entry ]) => {
        clearTimeout(timeout)
        timeout = setTimeout(() => {
          app.handle_resize(entry.contentRect.width , entry.contentRect.height )
        }, 50)
      })

      resizeObserver.observe(canvas)

      animate()
    },
  }
};