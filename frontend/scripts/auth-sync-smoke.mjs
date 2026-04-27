import { chromium } from 'playwright';

const frontendUrl = process.env.POP_TAIL_FRONTEND_URL || 'http://127.0.0.1:5666/';

async function waitForDashboard(page, label) {
  try {
    await page.waitForURL((url) => !url.pathname.startsWith('/auth/'), {
      timeout: 20_000,
    });
  } catch {
    throw new Error(`${label} did not reach dashboard, current url: ${page.url()}`);
  }
}

async function waitForLogin(page, label) {
  try {
    await page.waitForURL((url) => url.pathname === '/auth/login', {
      timeout: 20_000,
    });
  } catch {
    throw new Error(`${label} did not return to login, current url: ${page.url()}`);
  }
}

async function loginViaStore(page) {
  await page.goto(`${frontendUrl}auth/login`, { waitUntil: 'networkidle' });
  await page.evaluate(async () => {
    const authModule = await import('/src/store/pop-tail/auth.ts');
    const authStore = authModule.useAuthStore();
    await authStore.login('admin', '123456');
    const routerModule = await import('/src/router/index.ts');
    await routerModule.router.push('/dashboard');
  });
  await waitForDashboard(page, 'login tab');
}

async function main() {
  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext();

  const [pageA, pageB] = await Promise.all([context.newPage(), context.newPage()]);
  await Promise.all([loginViaStore(pageA), loginViaStore(pageB)]);

  await pageA.evaluate(async () => {
    const authModule = await import('/src/store/auth.ts');
    const authStore = authModule.useAuthStore();
    await authStore.logout();
    const routerModule = await import('/src/router/index.ts');
    await routerModule.router.replace('/auth/login');
  });

  await Promise.all([
    waitForLogin(pageA, 'pageA after logout'),
    waitForLogin(pageB, 'pageB after sync logout'),
  ]);

  console.log(
    JSON.stringify(
      {
        pageA: pageA.url(),
        pageB: pageB.url(),
        result: 'multi-tab logout sync ok',
      },
      null,
      2,
    ),
  );

  await browser.close();
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
});
