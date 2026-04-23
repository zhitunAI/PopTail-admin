import { defineConfig } from '@vben/vite-config';

export default defineConfig(async () => {
  return {
    application: {},
    vite: {
      server: {
        host: '127.0.0.1',
        port: 5666,
        strictPort: true,
        hmr: {
          host: '127.0.0.1',
          clientPort: 5666,
          port: 5666,
          protocol: 'ws',
          overlay: false,
        },
        proxy: {
          '/api': {
            changeOrigin: true,
            rewrite: (path) => path.replace(/^\/api/, ''),
            // mock代理目标地址
            target: 'http://localhost:5320/api',
            ws: true,
          },
        },
      },
    },
  };
});
