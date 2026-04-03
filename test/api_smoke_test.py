#!/usr/bin/env python3
"""API smoke test for the Rust Todo backend.

Usage examples:
  python3 test/api_smoke_test.py
  python3 test/api_smoke_test.py -d
  python3 test/api_smoke_test.py --base-url http://127.0.0.1:5050
  python3 test/api_smoke_test.py --start-server

Notes:
- Requires MongoDB at mongodb://127.0.0.1:27017
- If --start-server is not used, the backend must already be running.
"""

from __future__ import annotations

import argparse
import json
import random
import string
import subprocess
import sys
import time
from pathlib import Path
from typing import Any, Optional, Tuple
from urllib import error, parse, request


DEFAULT_BASE_URL = "http://127.0.0.1:5050"
USE_COLOR = sys.stdout.isatty()


class Ansi:
    RESET = "\033[0m"
    CYAN = "\033[36m"
    GREEN = "\033[32m"
    YELLOW = "\033[33m"
    RED = "\033[31m"


def _colorize(text: str, color: str, enabled: bool) -> str:
    if not enabled:
        return text
    return f"{color}{text}{Ansi.RESET}"


class TestFailure(Exception):
    pass


class ApiClient:
    def __init__(
        self,
        base_url: str,
        timeout_seconds: float = 10.0,
        debug: bool = False,
        color: bool = True,
    ) -> None:
        self.base_url = base_url.rstrip("/")
        self.timeout_seconds = timeout_seconds
        self.debug = debug
        self.color = color

    def request_json(
        self,
        method: str,
        path: str,
        payload: Optional[dict[str, Any]] = None,
        query: Optional[dict[str, Any]] = None,
        trace: bool = True,
    ) -> Tuple[int, Any]:
        method = method.upper()
        url = f"{self.base_url}{path}"
        if query:
            url = f"{url}?{parse.urlencode(query)}"

        headers = {
            "Accept": "application/json",
            "Content-Type": "application/json",
        }
        data = None
        if payload is not None:
            data = json.dumps(payload).encode("utf-8")

        if self.debug and trace:
            self._print_debug_request(method, url, payload)

        req = request.Request(url=url, data=data, headers=headers, method=method)

        try:
            with request.urlopen(req, timeout=self.timeout_seconds) as resp:
                raw = resp.read().decode("utf-8")
                body = _parse_json_or_text(raw)
                if self.debug and trace:
                    self._print_debug_response(resp.status, body)
                return resp.status, body
        except error.HTTPError as exc:
            raw = exc.read().decode("utf-8")
            body = _parse_json_or_text(raw)
            if self.debug and trace:
                self._print_debug_response(exc.code, body)
            return exc.code, body

    def _print_debug_request(
        self,
        method: str,
        url: str,
        payload: Optional[dict[str, Any]],
    ) -> None:
        label = _colorize("[DEBUG REQUEST]", Ansi.CYAN, self.color)
        print(f"{label} {method} {url}")
        print("Request JSON:")
        print(_pretty_json(payload))

    def _print_debug_response(self, status: int, body: Any) -> None:
        response_color = Ansi.GREEN if status < 400 else Ansi.RED
        label = _colorize("[DEBUG RESPONSE]", response_color, self.color)
        print(f"{label} HTTP {status}")
        print("Response JSON:")
        print(_pretty_json(body))
        print()


def _parse_json_or_text(raw: str) -> Any:
    if not raw:
        return None
    try:
        return json.loads(raw)
    except json.JSONDecodeError:
        return raw


def _pretty_json(value: Any) -> str:
    try:
        return json.dumps(value, indent=2, sort_keys=True)
    except TypeError:
        return str(value)


def _wait_for_server(
    client: ApiClient,
    timeout_seconds: int = 45,
    server_proc: Optional[subprocess.Popen[str]] = None,
) -> None:
    deadline = time.time() + timeout_seconds
    while time.time() < deadline:
        if server_proc is not None and server_proc.poll() is not None:
            output = ""
            if server_proc.stdout is not None:
                output = server_proc.stdout.read().strip()
            details = f"\nServer output:\n{output}" if output else ""
            raise TestFailure(
                "Backend process exited before becoming ready."
                + details
            )
        try:
            status, _ = client.request_json(
                "GET",
                "/get-user",
                query={"email": "_probe_"},
                trace=False,
            )
            if status == 200:
                return
        except Exception:
            pass
        time.sleep(1)
    raise TestFailure(
        f"Server at {client.base_url} was not reachable within {timeout_seconds}s"
    )


def _start_server(project_root: Path) -> subprocess.Popen[str]:
    cmd = ["cargo", "run", "--quiet"]
    proc = subprocess.Popen(
        cmd,
        cwd=project_root,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1,
    )
    return proc


def _stop_server(proc: subprocess.Popen[str]) -> None:
    if proc.poll() is not None:
        return

    proc.terminate()
    try:
        proc.wait(timeout=8)
    except subprocess.TimeoutExpired:
        proc.kill()
        proc.wait(timeout=5)


def _assert(condition: bool, message: str) -> None:
    if not condition:
        raise TestFailure(message)


def _step(name: str) -> None:
    print(f"{_colorize('[STEP]', Ansi.YELLOW, USE_COLOR)} {name}")


def _ok(name: str) -> None:
    print(f"{_colorize('[PASS]', Ansi.GREEN, USE_COLOR)} {name}")


def _get_task_by_id(tasks: list[dict[str, Any]], task_id: int) -> dict[str, Any]:
    for task in tasks:
        if task.get("id") == task_id:
            return task
    raise TestFailure(f"Task id={task_id} not found in tasks payload: {tasks}")


def run_smoke_test(client: ApiClient) -> None:
    suffix = f"{int(time.time())}_{_rand(6)}"
    username = f"test_user_{suffix}"
    email = f"todo.{suffix}@example.com"
    password = f"Pass_{suffix}!"

    _step("Create a new user")
    status, body = client.request_json(
        "POST",
        "/create-user",
        {
            "username": username,
            "email": email,
            "password": password,
        },
    )
    _assert(status == 200, f"/create-user expected HTTP 200, got {status}, body={body}")
    _assert(isinstance(body, dict), f"/create-user expected JSON object, got {body}")
    _assert(body.get("status") == "Success", f"/create-user failed: body={body}")
    _ok("User created")

    _step("Fetch user by email")
    status, body = client.request_json("GET", "/get-user", query={"email": email})
    _assert(status == 200, f"/get-user expected HTTP 200, got {status}, body={body}")
    _assert(body.get("email") == email, f"/get-user email mismatch: body={body}")
    _assert(body.get("username") == username, f"/get-user username mismatch: body={body}")
    _ok("User fetched")

    _step("Authenticate with wrong password (negative test)")
    status, body = client.request_json(
        "POST",
        "/auth",
        {
            "email": email,
            "pass": f"{password}_wrong",
        },
    )
    _assert(status == 200, f"/auth (wrong pass) expected HTTP 200, got {status}, body={body}")
    _assert(body.get("auth") is False, f"/auth (wrong pass) auth should be false: body={body}")
    _assert(body.get("error") is True, f"/auth (wrong pass) error should be true: body={body}")
    _assert(body.get("jwt") is None, f"/auth (wrong pass) jwt should be null: body={body}")
    _ok("Wrong password rejected")

    _step("Authenticate with valid password")
    status, body = client.request_json(
        "POST",
        "/auth",
        {
            "email": email,
            "pass": password,
        },
    )
    _assert(status == 200, f"/auth expected HTTP 200, got {status}, body={body}")
    _assert(body.get("auth") is True, f"/auth should return auth=true: body={body}")
    _assert(body.get("error") is False, f"/auth should return error=false: body={body}")
    jwt = body.get("jwt")
    _assert(isinstance(jwt, str) and len(jwt) > 10, f"/auth jwt missing/invalid: body={body}")
    _ok("Authentication succeeded")

    _step("Verify valid JWT")
    status, body = client.request_json("POST", "/jwt-auth", {"jwt": jwt})
    _assert(status == 200, f"/jwt-auth expected HTTP 200, got {status}, body={body}")
    _assert(body.get("tampered") is False, f"/jwt-auth tampered should be false: body={body}")
    _assert(body.get("email") == email, f"/jwt-auth email mismatch: body={body}")
    _ok("Valid JWT accepted")

    _step("Verify tampered JWT (negative test)")
    status, body = client.request_json("POST", "/jwt-auth", {"jwt": f"{jwt}.broken"})
    _assert(status == 200, f"/jwt-auth (tampered) expected HTTP 200, got {status}, body={body}")
    _assert(body.get("tampered") is True, f"/jwt-auth (tampered) should be true: body={body}")
    _assert(body.get("email") is None, f"/jwt-auth (tampered) email should be null: body={body}")
    _ok("Tampered JWT rejected")

    _step("Add two tasks")
    for idx, name in enumerate(("Write tests", "Ship release"), start=1):
        status, body = client.request_json(
            "POST",
            "/add-task",
            {
                "name": name,
                "priority": idx,
                "auth_jwt": jwt,
            },
        )
        _assert(status == 200, f"/add-task expected HTTP 200, got {status}, body={body}")
        _assert(body is True, f"/add-task expected true, got body={body}")
    _ok("Tasks added")

    _step("Get tasks with authenticated request")
    status, body = client.request_json(
        "GET",
        "/get-tasks",
        {
            "auth_jwt": jwt,
            "counter": 10,
        },
    )
    _assert(status == 200, f"/get-tasks expected HTTP 200, got {status}, body={body}")
    _assert(body.get("res") is True, f"/get-tasks should return res=true: body={body}")
    tasks = body.get("tasks")
    _assert(isinstance(tasks, list), f"/get-tasks tasks should be a list: body={body}")
    _assert(len(tasks) >= 1, f"/get-tasks expected at least one task, got {body}")
    first_task = tasks[0]
    _assert("id" in first_task and "name" in first_task, f"Unexpected task shape: {first_task}")
    _ok("Tasks fetched")

    first_task_id = first_task["id"]
    initial_status = first_task.get("status")
    _assert(isinstance(initial_status, bool), f"Task status should be bool: {first_task}")

    _step("Toggle first task status by id")
    status, body = client.request_json(
        "POST",
        "/update-task",
        {
            "id": first_task_id,
            "auth_jwt": jwt,
        },
    )
    _assert(status == 200, f"/update-task expected HTTP 200, got {status}, body={body}")
    _assert(body.get("success") is True, f"/update-task expected success=true, got {body}")
    _ok("Task status toggled once")

    _step("Verify first toggle changed task status")
    status, body = client.request_json(
        "GET",
        "/get-tasks",
        {
            "auth_jwt": jwt,
            "counter": 10,
        },
    )
    _assert(status == 200, f"/get-tasks (after 1st toggle) expected HTTP 200, got {status}, body={body}")
    _assert(body.get("res") is True, f"/get-tasks (after 1st toggle) should return res=true: body={body}")
    tasks = body.get("tasks")
    _assert(isinstance(tasks, list), f"/get-tasks (after 1st toggle) tasks should be list: body={body}")
    toggled_task = _get_task_by_id(tasks, first_task_id)
    _assert(
        toggled_task.get("status") is (not initial_status),
        f"/update-task should toggle status from {initial_status} to {not initial_status}, got {toggled_task}",
    )
    _ok("First toggle verified")

    _step("Toggle first task status back to original value")
    status, body = client.request_json(
        "POST",
        "/update-task",
        {
            "id": first_task_id,
            "auth_jwt": jwt,
        },
    )
    _assert(status == 200, f"/update-task (2nd toggle) expected HTTP 200, got {status}, body={body}")
    _assert(body.get("success") is True, f"/update-task (2nd toggle) expected success=true, got {body}")
    _ok("Task status toggled twice")

    _step("Verify second toggle restored original task status")
    status, body = client.request_json(
        "GET",
        "/get-tasks",
        {
            "auth_jwt": jwt,
            "counter": 10,
        },
    )
    _assert(status == 200, f"/get-tasks (after 2nd toggle) expected HTTP 200, got {status}, body={body}")
    _assert(body.get("res") is True, f"/get-tasks (after 2nd toggle) should return res=true: body={body}")
    tasks = body.get("tasks")
    _assert(isinstance(tasks, list), f"/get-tasks (after 2nd toggle) tasks should be list: body={body}")
    toggled_back_task = _get_task_by_id(tasks, first_task_id)
    _assert(
        toggled_back_task.get("status") is initial_status,
        f"/update-task second toggle should restore status={initial_status}, got {toggled_back_task}",
    )
    _ok("Second toggle verified")

    _step("Update unknown task id (negative test)")
    status, body = client.request_json(
        "POST",
        "/update-task",
        {
            "id": first_task_id + 10_000_000,
            "auth_jwt": jwt,
        },
    )
    _assert(
        status == 500,
        f"/update-task (unknown task id) expected HTTP 500, got {status}, body={body}",
    )
    _assert(
        body.get("success") is False,
        f"/update-task (unknown task id) should return success=false, got {body}",
    )
    _ok("Unknown task id rejected")

    _step("Delete task with invalid JWT (negative test)")
    status, body = client.request_json(
        "POST",
        "/delete-task",
        {
            "auth_jwt": f"{jwt}.broken",
            "task_id": first_task_id,
        },
    )
    _assert(status == 500, f"/delete-task (invalid jwt) expected HTTP 500, got {status}, body={body}")
    _assert(body.get("success") is False, f"/delete-task (invalid jwt) should fail: body={body}")
    _ok("Invalid-JWT delete rejected")

    _step("Delete task with valid JWT")
    status, body = client.request_json(
        "POST",
        "/delete-task",
        {
            "auth_jwt": jwt,
            "task_id": first_task_id,
        },
    )
    _assert(
        status == 200,
        f"/delete-task (correct pass) expected HTTP 200, got {status}, body={body}",
    )
    _assert(body.get("success") is True, f"/delete-task should return success=true: body={body}")
    _ok("Task deleted")


def _rand(length: int) -> str:
    alphabet = string.ascii_lowercase + string.digits
    return "".join(random.choice(alphabet) for _ in range(length))


def main() -> int:
    parser = argparse.ArgumentParser(description="Smoke test the Todo backend API")
    parser.add_argument("--base-url", default=DEFAULT_BASE_URL, help="API base URL")
    parser.add_argument(
        "-d",
        "--debug",
        action="store_true",
        help="Print colorized request/response JSON for each API call",
    )
    parser.add_argument(
        "--start-server",
        action="store_true",
        help="Start backend with `cargo run --quiet` before tests",
    )
    args = parser.parse_args()

    client = ApiClient(args.base_url, debug=args.debug, color=USE_COLOR)
    server_proc: Optional[subprocess.Popen[str]] = None

    try:
        if args.start_server:
            project_root = Path(__file__).resolve().parent.parent
            print("[INFO] Starting backend server with cargo run --quiet")
            server_proc = _start_server(project_root)

        print(f"[INFO] Waiting for server: {args.base_url}")
        _wait_for_server(client, server_proc=server_proc)

        print("[INFO] Running smoke test flow")
        run_smoke_test(client)

        print("\n[SUCCESS] All smoke tests passed")
        return 0
    except TestFailure as exc:
        print(f"\n[FAIL] {exc}")
        return 1
    except Exception as exc:
        print(f"\n[ERROR] Unexpected exception: {exc}")
        return 1
    finally:
        if server_proc is not None:
            print("[INFO] Stopping backend server")
            _stop_server(server_proc)


if __name__ == "__main__":
    sys.exit(main())
