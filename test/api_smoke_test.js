#!/usr/bin/env bun

/**
 * API smoke test for the Rust Todo backend.
 *
 * Usage examples:
 *   bun test/api_smoke_test.js
 *   bun test/api_smoke_test.js -d
 *   bun test/api_smoke_test.js --base-url http://127.0.0.1:5050
 *   bun test/api_smoke_test.js --start-server
 *
 * Notes:
 * - Requires MongoDB at mongodb://127.0.0.1:27017
 * - If --start-server is not used, the backend must already be running.
 */

import { spawn } from "node:child_process";
import http from "node:http";
import https from "node:https";
import path from "node:path";
import process from "node:process";
import { setTimeout as sleep } from "node:timers/promises";
import { fileURLToPath } from "node:url";

const DEFAULT_BASE_URL = "http://127.0.0.1:5050";
const USE_COLOR = Boolean(process.stdout.isTTY);

const Ansi = {
  RESET: "\x1b[0m",
  CYAN: "\x1b[36m",
  GREEN: "\x1b[32m",
  YELLOW: "\x1b[33m",
  RED: "\x1b[31m",
};

class TestFailure extends Error {}

function colorize(text, color, enabled) {
  return enabled ? `${color}${text}${Ansi.RESET}` : text;
}

function parseJsonOrText(raw) {
  if (!raw) {
    return null;
  }

  try {
    return JSON.parse(raw);
  } catch {
    return raw;
  }
}

function prettyJson(value) {
  try {
    return JSON.stringify(value, null, 2);
  } catch {
    return String(value);
  }
}

function assert(condition, message) {
  if (!condition) {
    throw new TestFailure(message);
  }
}

function step(name) {
  console.log(`${colorize("[STEP]", Ansi.YELLOW, USE_COLOR)} ${name}`);
}

function ok(name) {
  console.log(`${colorize("[PASS]", Ansi.GREEN, USE_COLOR)} ${name}`);
}

function rand(length) {
  const alphabet = "abcdefghijklmnopqrstuvwxyz0123456789";
  let out = "";
  for (let i = 0; i < length; i += 1) {
    out += alphabet[Math.floor(Math.random() * alphabet.length)];
  }
  return out;
}

function getTaskById(tasks, taskId) {
  const task = tasks.find((t) => t?.id === taskId);
  if (!task) {
    throw new TestFailure(`Task id=${taskId} not found in tasks payload: ${prettyJson(tasks)}`);
  }
  return task;
}

function getProjectRoot() {
  const scriptDir = path.dirname(fileURLToPath(import.meta.url));
  return path.resolve(scriptDir, "..");
}

class ApiClient {
  constructor(baseUrl, timeoutMs = 10_000, debug = false, color = true) {
    this.baseUrl = baseUrl.replace(/\/$/, "");
    this.timeoutMs = timeoutMs;
    this.debug = debug;
    this.color = color;
  }

  async requestJson(method, path, payload = null, query = null, trace = true) {
    const upperMethod = method.toUpperCase();
    const url = new URL(`${this.baseUrl}${path}`);

    if (query) {
      for (const [key, value] of Object.entries(query)) {
        url.searchParams.set(key, String(value));
      }
    }

    if (this.debug && trace) {
      this.printDebugRequest(upperMethod, url.toString(), payload);
    }

    // fetch() does not allow GET requests with a body, but this API's
    // /get-tasks endpoint expects JSON payload on GET.
    if (upperMethod === "GET" && payload !== null) {
      const [status, body] = await this.requestViaNodeHttp(upperMethod, url, payload);
      if (this.debug && trace) {
        this.printDebugResponse(status, body);
      }
      return [status, body];
    }

    const controller = new AbortController();
    const timeout = setTimeout(() => controller.abort(), this.timeoutMs);

    try {
      const response = await fetch(url, {
        method: upperMethod,
        headers: {
          Accept: "application/json",
          "Content-Type": "application/json",
        },
        body: payload === null ? undefined : JSON.stringify(payload),
        signal: controller.signal,
      });

      const raw = await response.text();
      const body = parseJsonOrText(raw);

      if (this.debug && trace) {
        this.printDebugResponse(response.status, body);
      }

      return [response.status, body];
    } catch (error) {
      if (error instanceof Error && error.name === "AbortError") {
        throw new TestFailure(`Request timed out after ${this.timeoutMs}ms: ${upperMethod} ${url}`);
      }
      throw error;
    } finally {
      clearTimeout(timeout);
    }
  }

  requestViaNodeHttp(method, url, payload) {
    const transport = url.protocol === "https:" ? https : http;
    const bodyString = payload === null ? "" : JSON.stringify(payload);
    const headers = {
      Accept: "application/json",
      "Content-Type": "application/json",
    };

    if (bodyString) {
      headers["Content-Length"] = Buffer.byteLength(bodyString).toString();
    }

    return new Promise((resolve, reject) => {
      const req = transport.request(
        url,
        {
          method,
          headers,
        },
        (res) => {
          let raw = "";
          res.setEncoding("utf8");
          res.on("data", (chunk) => {
            raw += chunk;
          });
          res.on("end", () => {
            resolve([res.statusCode ?? 0, parseJsonOrText(raw)]);
          });
        },
      );

      req.setTimeout(this.timeoutMs, () => {
        req.destroy(new TestFailure(`Request timed out after ${this.timeoutMs}ms: ${method} ${url}`));
      });
      req.on("error", reject);

      if (bodyString) {
        req.write(bodyString);
      }
      req.end();
    });
  }

  printDebugRequest(method, url, payload) {
    const label = colorize("[DEBUG REQUEST]", Ansi.CYAN, this.color);
    console.log(`${label} ${method} ${url}`);
    console.log("Request JSON:");
    console.log(prettyJson(payload));
  }

  printDebugResponse(status, body) {
    const responseColor = status < 400 ? Ansi.GREEN : Ansi.RED;
    const label = colorize("[DEBUG RESPONSE]", responseColor, this.color);
    console.log(`${label} HTTP ${status}`);
    console.log("Response JSON:");
    console.log(prettyJson(body));
    console.log();
  }
}

function startServer(projectRoot) {
  const proc = spawn("cargo", ["run", "--quiet"], {
    cwd: projectRoot,
    stdio: ["ignore", "pipe", "pipe"],
  });

  let logs = "";
  proc.stdout.setEncoding("utf8");
  proc.stderr.setEncoding("utf8");
  proc.stdout.on("data", (chunk) => {
    logs += chunk;
  });
  proc.stderr.on("data", (chunk) => {
    logs += chunk;
  });

  return { proc, getLogs: () => logs.trim() };
}

async function stopServer(proc) {
  if (proc.exitCode !== null) {
    return;
  }

  proc.kill("SIGTERM");

  const exited = await Promise.race([
    new Promise((resolve) => {
      proc.once("exit", () => resolve(true));
    }),
    sleep(8_000).then(() => false),
  ]);

  if (!exited) {
    proc.kill("SIGKILL");
    await new Promise((resolve) => {
      proc.once("exit", () => resolve());
    });
  }
}

async function waitForServer(client, timeoutSeconds = 45, serverHandle = null) {
  const deadline = Date.now() + timeoutSeconds * 1000;

  while (Date.now() < deadline) {
    if (serverHandle && serverHandle.proc.exitCode !== null) {
      const output = serverHandle.getLogs();
      const details = output ? `\nServer output:\n${output}` : "";
      throw new TestFailure(`Backend process exited before becoming ready.${details}`);
    }

    try {
      const [status] = await client.requestJson(
        "GET",
        "/get-user",
        null,
        { email: "_probe_" },
        false,
      );
      if (status === 200) {
        return;
      }
    } catch {
      // Retry until timeout.
    }

    await sleep(1_000);
  }

  throw new TestFailure(`Server at ${client.baseUrl} was not reachable within ${timeoutSeconds}s`);
}

async function runSmokeTest(client) {
  const suffix = `${Math.floor(Date.now() / 1000)}_${rand(6)}`;
  const username = `test_user_${suffix}`;
  const email = `todo.${suffix}@example.com`;
  const password = `Pass_${suffix}!`;

  step("Create a new user");
  let [status, body] = await client.requestJson("POST", "/create-user", {
    username,
    email,
    password,
  });
  assert(status === 200, `/create-user expected HTTP 200, got ${status}, body=${prettyJson(body)}`);
  assert(body && typeof body === "object", `/create-user expected JSON object, got ${prettyJson(body)}`);
  assert(body.status === "Success", `/create-user failed: body=${prettyJson(body)}`);
  ok("User created");

  step("Fetch user by email");
  [status, body] = await client.requestJson("GET", "/get-user", null, { email });
  assert(status === 200, `/get-user expected HTTP 200, got ${status}, body=${prettyJson(body)}`);
  assert(body?.email === email, `/get-user email mismatch: body=${prettyJson(body)}`);
  assert(body?.username === username, `/get-user username mismatch: body=${prettyJson(body)}`);
  ok("User fetched");

  step("Authenticate with wrong password (negative test)");
  [status, body] = await client.requestJson("POST", "/auth", {
    email,
    pass: `${password}_wrong`,
  });
  assert(status === 200, `/auth (wrong pass) expected HTTP 200, got ${status}, body=${prettyJson(body)}`);
  assert(body?.auth === false, `/auth (wrong pass) auth should be false: body=${prettyJson(body)}`);
  assert(body?.error === true, `/auth (wrong pass) error should be true: body=${prettyJson(body)}`);
  assert(body?.jwt === null, `/auth (wrong pass) jwt should be null: body=${prettyJson(body)}`);
  ok("Wrong password rejected");

  step("Authenticate with valid password");
  [status, body] = await client.requestJson("POST", "/auth", {
    email,
    pass: password,
  });
  assert(status === 200, `/auth expected HTTP 200, got ${status}, body=${prettyJson(body)}`);
  assert(body?.auth === true, `/auth should return auth=true: body=${prettyJson(body)}`);
  assert(body?.error === false, `/auth should return error=false: body=${prettyJson(body)}`);
  const jwt = body?.jwt;
  assert(typeof jwt === "string" && jwt.length > 10, `/auth jwt missing/invalid: body=${prettyJson(body)}`);
  ok("Authentication succeeded");

  step("Verify valid JWT");
  [status, body] = await client.requestJson("POST", "/jwt-auth", { jwt });
  assert(status === 200, `/jwt-auth expected HTTP 200, got ${status}, body=${prettyJson(body)}`);
  assert(body?.tampered === false, `/jwt-auth tampered should be false: body=${prettyJson(body)}`);
  assert(body?.email === email, `/jwt-auth email mismatch: body=${prettyJson(body)}`);
  ok("Valid JWT accepted");

  step("Verify tampered JWT (negative test)");
  [status, body] = await client.requestJson("POST", "/jwt-auth", { jwt: `${jwt}.broken` });
  assert(status === 200, `/jwt-auth (tampered) expected HTTP 200, got ${status}, body=${prettyJson(body)}`);
  assert(body?.tampered === true, `/jwt-auth (tampered) should be true: body=${prettyJson(body)}`);
  assert(body?.email === null, `/jwt-auth (tampered) email should be null: body=${prettyJson(body)}`);
  ok("Tampered JWT rejected");

  step("Add two tasks");
  for (const [idx, name] of ["Write tests", "Ship release"].entries()) {
    [status, body] = await client.requestJson("POST", "/add-task", {
      name,
      priority: idx + 1,
      auth_jwt: jwt,
    });
    assert(status === 200, `/add-task expected HTTP 200, got ${status}, body=${prettyJson(body)}`);
    assert(body === true, `/add-task expected true, got body=${prettyJson(body)}`);
  }
  ok("Tasks added");

  step("Get tasks with authenticated request");
  [status, body] = await client.requestJson("GET", "/get-tasks", {
    auth_jwt: jwt,
    counter: 10,
  });
  assert(status === 200, `/get-tasks expected HTTP 200, got ${status}, body=${prettyJson(body)}`);
  assert(body?.res === true, `/get-tasks should return res=true: body=${prettyJson(body)}`);
  let tasks = body?.tasks;
  assert(Array.isArray(tasks), `/get-tasks tasks should be a list: body=${prettyJson(body)}`);
  assert(tasks.length >= 1, `/get-tasks expected at least one task, got ${prettyJson(body)}`);
  const firstTask = tasks[0];
  assert("id" in firstTask && "name" in firstTask, `Unexpected task shape: ${prettyJson(firstTask)}`);
  ok("Tasks fetched");

  const firstTaskId = firstTask.id;
  const initialStatus = firstTask.status;
  assert(typeof initialStatus === "boolean", `Task status should be bool: ${prettyJson(firstTask)}`);

  step("Toggle first task status by id");
  [status, body] = await client.requestJson("POST", "/update-task", {
    id: firstTaskId,
    auth_jwt: jwt,
  });
  assert(status === 200, `/update-task expected HTTP 200, got ${status}, body=${prettyJson(body)}`);
  assert(body?.success === true, `/update-task expected success=true, got ${prettyJson(body)}`);
  ok("Task status toggled once");

  step("Verify first toggle changed task status");
  [status, body] = await client.requestJson("GET", "/get-tasks", {
    auth_jwt: jwt,
    counter: 10,
  });
  assert(status === 200, `/get-tasks (after 1st toggle) expected HTTP 200, got ${status}, body=${prettyJson(body)}`);
  assert(body?.res === true, `/get-tasks (after 1st toggle) should return res=true: body=${prettyJson(body)}`);
  tasks = body?.tasks;
  assert(Array.isArray(tasks), `/get-tasks (after 1st toggle) tasks should be list: body=${prettyJson(body)}`);
  const toggledTask = getTaskById(tasks, firstTaskId);
  assert(
    toggledTask.status === !initialStatus,
    `/update-task should toggle status from ${initialStatus} to ${!initialStatus}, got ${prettyJson(toggledTask)}`,
  );
  ok("First toggle verified");

  step("Toggle first task status back to original value");
  [status, body] = await client.requestJson("POST", "/update-task", {
    id: firstTaskId,
    auth_jwt: jwt,
  });
  assert(status === 200, `/update-task (2nd toggle) expected HTTP 200, got ${status}, body=${prettyJson(body)}`);
  assert(body?.success === true, `/update-task (2nd toggle) expected success=true, got ${prettyJson(body)}`);
  ok("Task status toggled twice");

  step("Verify second toggle restored original task status");
  [status, body] = await client.requestJson("GET", "/get-tasks", {
    auth_jwt: jwt,
    counter: 10,
  });
  assert(status === 200, `/get-tasks (after 2nd toggle) expected HTTP 200, got ${status}, body=${prettyJson(body)}`);
  assert(body?.res === true, `/get-tasks (after 2nd toggle) should return res=true: body=${prettyJson(body)}`);
  tasks = body?.tasks;
  assert(Array.isArray(tasks), `/get-tasks (after 2nd toggle) tasks should be list: body=${prettyJson(body)}`);
  const toggledBackTask = getTaskById(tasks, firstTaskId);
  assert(
    toggledBackTask.status === initialStatus,
    `/update-task second toggle should restore status=${initialStatus}, got ${prettyJson(toggledBackTask)}`,
  );
  ok("Second toggle verified");

  step("Update unknown task id (negative test)");
  [status, body] = await client.requestJson("POST", "/update-task", {
    id: firstTaskId + 10_000_000,
    auth_jwt: jwt,
  });
  assert(
    status === 500,
    `/update-task (unknown task id) expected HTTP 500, got ${status}, body=${prettyJson(body)}`,
  );
  assert(
    body?.success === false,
    `/update-task (unknown task id) should return success=false, got ${prettyJson(body)}`,
  );
  ok("Unknown task id rejected");

  step("Delete task with invalid JWT (negative test)");
  [status, body] = await client.requestJson("POST", "/delete-task", {
    auth_jwt: `${jwt}.broken`,
    task_id: firstTaskId,
  });
  assert(status === 500, `/delete-task (invalid jwt) expected HTTP 500, got ${status}, body=${prettyJson(body)}`);
  assert(body?.success === false, `/delete-task (invalid jwt) should fail: body=${prettyJson(body)}`);
  ok("Invalid-JWT delete rejected");

  step("Delete task with valid JWT");
  [status, body] = await client.requestJson("POST", "/delete-task", {
    auth_jwt: jwt,
    task_id: firstTaskId,
  });
  assert(status === 200, `/delete-task expected HTTP 200, got ${status}, body=${prettyJson(body)}`);
  assert(body?.success === true, `/delete-task should return success=true: body=${prettyJson(body)}`);
  ok("Task deleted");
}

function parseArgs(argv) {
  const args = {
    baseUrl: DEFAULT_BASE_URL,
    debug: false,
    startServer: false,
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];

    if (arg === "-d" || arg === "--debug") {
      args.debug = true;
      continue;
    }

    if (arg === "--start-server") {
      args.startServer = true;
      continue;
    }

    if (arg === "--base-url") {
      const value = argv[i + 1];
      if (!value) {
        throw new TestFailure("Missing value for --base-url");
      }
      args.baseUrl = value;
      i += 1;
      continue;
    }

    if (arg === "-h" || arg === "--help") {
      printHelp();
      process.exit(0);
    }

    throw new TestFailure(`Unknown argument: ${arg}`);
  }

  return args;
}

function printHelp() {
  console.log(`Smoke test the Todo backend API

Usage:
  bun test/api_smoke_test.js [options]

Options:
  --base-url <url>    API base URL (default: ${DEFAULT_BASE_URL})
  -d, --debug         Print colorized request/response JSON for each API call
  --start-server      Start backend with cargo run --quiet before tests
  -h, --help          Show this help message
`);
}

async function main() {
  const args = parseArgs(process.argv.slice(2));
  const client = new ApiClient(args.baseUrl, 10_000, args.debug, USE_COLOR);

  /** @type {{ proc: import("node:child_process").ChildProcessWithoutNullStreams, getLogs: () => string } | null} */
  let serverHandle = null;

  try {
    if (args.startServer) {
      console.log("[INFO] Starting backend server with cargo run --quiet");
      serverHandle = startServer(getProjectRoot());
    }

    console.log(`[INFO] Waiting for server: ${args.baseUrl}`);
    await waitForServer(client, 45, serverHandle);

    console.log("[INFO] Running smoke test flow");
    await runSmokeTest(client);

    console.log("\n[SUCCESS] All smoke tests passed");
    process.exitCode = 0;
  } catch (error) {
    if (error instanceof TestFailure) {
      console.error(`\n[FAIL] ${error.message}`);
    } else {
      const message = error instanceof Error ? error.message : String(error);
      console.error(`\n[ERROR] Unexpected exception: ${message}`);
    }
    process.exitCode = 1;
  } finally {
    if (serverHandle) {
      console.log("[INFO] Stopping backend server");
      await stopServer(serverHandle.proc);
    }
  }
}

await main();
