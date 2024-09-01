import { defineConfig } from 'vite';
import { visualizer } from 'rollup-plugin-visualizer';
import react from '@vitejs/plugin-react';
import svgr from 'vite-plugin-svgr';

// https://vitejs.dev/config/
export default defineConfig({
    plugins: [svgr(), react(), visualizer({ template: 'network' })],
    // build: {
    //     commonjsOptions: { include: [] },
    // },
    // optimizeDeps: {
    //     disabled: false,
    // },
});
