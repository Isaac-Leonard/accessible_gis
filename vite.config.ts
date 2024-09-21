import { defineConfig } from "vite";
import prefresh from "@prefresh/vite";

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  build: {
    target: "esnext",
  },
  esbuild: {
    jsxFactory: "h",
    jsxFragment: "Fragment",
    jsxInject: `import { h, Fragment } from 'preact'`,
  },
  plugins: [prefresh()],
  resolve: {
    alias: {
      react: "preact/compat",
    },
  },
  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      // 3. tell vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**", "**/external-touch-device/**"],
    },
  },
}));
