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

      let mousedown = false

      function randomize(e) {
        const rect = canvas.getBoundingClientRect();

        const x = e.clientX - rect.left;
        const y = e.clientY - rect.top;

        app.randomize(
          x * (canvas.width / rect.width),
          y * (canvas.height / rect.height)
        )
      }

      canvas.addEventListener("mousedown", (e) => {
        mousedown = true
        randomize(e)
      })
      canvas.addEventListener("mouseup", () => {
        mousedown = false
      })

      canvas.addEventListener("mousemove", (e) => {
        if(!mousedown) {
          return
        }
        randomize(e)
      })

      function animate() {
        app.render_frame(playing)
        requestAnimationFrame(animate)
      }

      let timeout = null;

      let resizeObserver = new ResizeObserver(([ entry ]) => {
        clearTimeout(timeout)
        timeout = setTimeout(() => {
          app.handle_resize(entry.contentRect.width / 3, entry.contentRect.height / 3)
        }, 50)
      })

      resizeObserver.observe(canvas)

      animate()
      app.render_frame(playing)
    },
  }
};