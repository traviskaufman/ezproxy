import { describe, test, expect, afterEach } from "vitest";
import { execSync } from "node:child_process";
import { writeFileSync, unlinkSync, existsSync } from "node:fs";
import { homedir } from "node:os";
import { join } from "node:path";
import http from "node:http";

const CONFIG_PATH = join(homedir(), "ezproxy.txt");
const PROXY_PORT = 5050;
const PROXY_BASE = `http://localhost:${PROXY_PORT}`;
const APP_NAME = "EZProxyOSX";

const APP_STARTUP_DELAY_MS = 2000;
const CONFIG_RELOAD_DELAY_MS = 1500;

function httpGet(url: string): Promise<http.IncomingMessage> {
  return new Promise((resolve, reject) => {
    const req = http.get(url, (res) => resolve(res));
    req.on("error", reject);
    req.setTimeout(5000, () => {
      req.destroy();
      reject(new Error("Request timed out"));
    });
  });
}

async function waitForServer(
  timeoutMs: number = APP_STARTUP_DELAY_MS
): Promise<void> {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    try {
      await httpGet(`${PROXY_BASE}/?q=ping`);
      return;
    } catch {
      await new Promise((r) => setTimeout(r, 200));
    }
  }
  throw new Error(`Server did not start within ${timeoutMs}ms`);
}

async function waitForServerDown(timeoutMs: number = 3000): Promise<void> {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    try {
      await httpGet(`${PROXY_BASE}/?q=ping`);
      await new Promise((r) => setTimeout(r, 200));
    } catch {
      return;
    }
  }
  throw new Error(`Server did not stop within ${timeoutMs}ms`);
}

function launchApp(): void {
  execSync(`open -a ${APP_NAME}`);
}

function quitApp(): void {
  try {
    execSync(`osascript -e 'quit app "${APP_NAME}"'`);
  } catch {
    try {
      execSync(`pkill -f ${APP_NAME}`);
    } catch {}
  }
}

function writeConfig(content: string): void {
  writeFileSync(CONFIG_PATH, content, "utf-8");
}

function removeConfig(): void {
  if (existsSync(CONFIG_PATH)) {
    unlinkSync(CONFIG_PATH);
  }
}

const TEST_CONFIG = `m = https://gmail.com/
cal = https://calendar.google.com/
npm = https://npmjs.com/search?q={ARGS}
_ = https://www.google.com/search?q={ALL}`;

describe("native-installer: Tier 1 Integration Tests", () => {
  afterEach(async () => {
    quitApp();
    await waitForServerDown();
    removeConfig();
  });

  test("starts an HTTP server on localhost:5050 that returns 302 redirects", async () => {
    writeConfig(TEST_CONFIG);
    launchApp();
    await waitForServer();

    const resp = await httpGet(`${PROXY_BASE}/?q=m`);

    expect(resp.statusCode).toBe(302);
    expect(resp.headers["location"]).toBe("https://gmail.com/");
    expect(resp.headers["x-ez-made-this"]).toBe("true");
  });

  test("redirects with {ARGS} token substitution", async () => {
    writeConfig(TEST_CONFIG);
    launchApp();
    await waitForServer();

    const resp = await httpGet(`${PROXY_BASE}/?q=npm%20file%20finder`);

    expect(resp.statusCode).toBe(302);
    expect(resp.headers["location"]).toBe(
      "https://npmjs.com/search?q=file%20finder"
    );
  });

  test("redirects with {ALL} fallback rule substitution", async () => {
    writeConfig(TEST_CONFIG);
    launchApp();
    await waitForServer();

    const resp = await httpGet(
      `${PROXY_BASE}/?q=best%20restaurants%20nyc`
    );

    expect(resp.statusCode).toBe(302);
    expect(resp.headers["location"]).toBe(
      "https://www.google.com/search?q=best%20restaurants%20nyc"
    );
  });

  test("redirects to a simple URL with no token substitution", async () => {
    writeConfig(TEST_CONFIG);
    launchApp();
    await waitForServer();

    const resp = await httpGet(`${PROXY_BASE}/?q=cal`);

    expect(resp.statusCode).toBe(302);
    expect(resp.headers["location"]).toBe("https://calendar.google.com/");
  });

  test("runs without a config file and returns errors for all queries", async () => {
    removeConfig();
    launchApp();
    await waitForServer();

    const resp = await httpGet(`${PROXY_BASE}/?q=m`);

    expect(resp.statusCode).toBe(500);
  });

  test("reloads config transparently when ~/ezproxy.txt is modified", async () => {
    writeConfig("m = https://gmail.com/");
    launchApp();
    await waitForServer();

    const resp1 = await httpGet(`${PROXY_BASE}/?q=m`);
    expect(resp1.statusCode).toBe(302);
    expect(resp1.headers["location"]).toBe("https://gmail.com/");

    writeConfig("m = https://outlook.com/");
    await new Promise((r) => setTimeout(r, CONFIG_RELOAD_DELAY_MS));

    const resp2 = await httpGet(`${PROXY_BASE}/?q=m`);
    expect(resp2.statusCode).toBe(302);
    expect(resp2.headers["location"]).toBe("https://outlook.com/");
  });

  test("reloads config when file is created after app launch", async () => {
    removeConfig();
    launchApp();
    await waitForServer();

    const resp1 = await httpGet(`${PROXY_BASE}/?q=m`);
    expect(resp1.statusCode).toBe(500);

    writeConfig("m = https://gmail.com/");
    await new Promise((r) => setTimeout(r, CONFIG_RELOAD_DELAY_MS));

    const resp2 = await httpGet(`${PROXY_BASE}/?q=m`);
    expect(resp2.statusCode).toBe(302);
    expect(resp2.headers["location"]).toBe("https://gmail.com/");
  });

  test("clears rules when config file is deleted while app is running", async () => {
    writeConfig("m = https://gmail.com/");
    launchApp();
    await waitForServer();

    const resp1 = await httpGet(`${PROXY_BASE}/?q=m`);
    expect(resp1.statusCode).toBe(302);

    removeConfig();
    await new Promise((r) => setTimeout(r, CONFIG_RELOAD_DELAY_MS));

    const resp2 = await httpGet(`${PROXY_BASE}/?q=m`);
    expect(resp2.statusCode).toBe(500);
  });
});
