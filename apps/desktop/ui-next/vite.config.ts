import { svelte } from '@sveltejs/vite-plugin-svelte'
import { defineConfig } from 'vite'
import path from 'node:path'

// https://vite.dev/config/
export default defineConfig({
  plugins: [svelte()],
  
  // Tauri expects a relative base path
  base: './',
  
  clearScreen: false,
  
  resolve: {
    alias: {
      '@': path.resolve(import.meta.dirname, './src'),
      '@/lib': path.resolve(import.meta.dirname, './src/lib'),
      '@/stores': path.resolve(import.meta.dirname, './src/stores'),
      '@/components': path.resolve(import.meta.dirname, './src/components'),
      '@/types': path.resolve(import.meta.dirname, './src/types'),
    },
  },
  
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
  
  build: {
    target: 'es2021',
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_DEBUG,
    outDir: 'dist',
    
    // Bundle optimization
    rollupOptions: {
      output: {
        // Manual chunk splitting for better caching
        manualChunks: (id) => {
          // Vendor chunk for node_modules
          if (id.includes('node_modules')) {
            // Separate @tauri-apps (stable, rarely changes)
            if (id.includes('@tauri-apps')) {
              return 'vendor-tauri';
            }
            // Other vendors (svelte, etc)
            return 'vendor';
          }
          // App code stays in main chunk for now (POC is small)
        },
        
        // Clean output filenames
        entryFileNames: 'assets/[name]-[hash].js',
        chunkFileNames: 'assets/[name]-[hash].js',
        assetFileNames: 'assets/[name]-[hash][extname]',
      },
    },
    
    // Compress threshold (only compress files > 10kb)
    reportCompressedSize: true,
    chunkSizeWarningLimit: 1000, // Warn if chunk > 1000kb
  },
})
