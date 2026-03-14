import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

export default defineConfig({
  plugins: [vue()],
  server: {
    port: 5173,
    proxy: {
      // Proxy all /v1/* + /healthz + /version through Vite to avoid cross-port
      // EventSource restrictions in some browsers (Chrome, Edge).
      '/v1': {
        target: 'http://localhost:8080',
        changeOrigin: true,
        // Disable response buffering so SSE events flow through immediately.
        configure: (proxy) => {
          proxy.on('proxyReq', (_req, res) => {
            // @ts-ignore — node http.ServerResponse
            res.setHeader?.('X-Accel-Buffering', 'no')
          })
        },
      },
      '/healthz':  { target: 'http://localhost:8080', changeOrigin: true },
      '/version':  { target: 'http://localhost:8080', changeOrigin: true },
    },
  },
})
