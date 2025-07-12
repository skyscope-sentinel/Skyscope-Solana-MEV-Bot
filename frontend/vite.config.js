import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { resolve } from 'path';

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  
  // Vite options tailored for Tauri development
  // https://tauri.app/v1/guides/getting-started/setup/vite
  server: {
    port: 1420,
    strictPort: true,
    // Allow Tauri to make requests to the development server
    cors: true,
    hmr: {
      protocol: 'ws',
    },
  },
  
  // Resolve paths and aliases
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
      '@components': resolve(__dirname, 'src/components'),
      '@assets': resolve(__dirname, 'src/assets'),
      '@styles': resolve(__dirname, 'src/styles'),
    },
  },
  
  // Build configuration
  build: {
    // Tauri supports ES2021
    target: ['es2021', 'chrome100', 'safari13'],
    // Don't minify for debug builds
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    // Produce sourcemaps for debug builds
    sourcemap: !!process.env.TAURI_DEBUG,
    // Output directory (relative to project root)
    outDir: 'dist',
    // Empty outDir before building
    emptyOutDir: true,
  },
  
  // Optimizations
  optimizeDeps: {
    include: ['react', 'react-dom', '@tauri-apps/api'],
  },
  
  // Environment variables
  define: {
    'process.env.NODE_ENV': JSON.stringify(process.env.NODE_ENV || 'development'),
  },
});
