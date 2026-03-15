#!/usr/bin/env python3
"""Thin CLI for the local live EPU workbench HTTP API."""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
import time
from pathlib import Path
from typing import Any
from urllib import error, request


REPO_ROOT = Path(__file__).resolve().parents[1]
DEFAULT_HOST = "127.0.0.1"
DEFAULT_SESSION_FILE = REPO_ROOT / "tmp" / "epu-workbench" / "session.json"
DEFAULT_LAUNCH_FILE = DEFAULT_SESSION_FILE.parent / "launch.json"
API_PREFIX = "/api/epu-workbench"
PRESET_SOURCE = REPO_ROOT / "examples" / "3-inspectors" / "epu-showcase" / "src" / "presets.rs"
BENCHMARK_SOURCE = REPO_ROOT / "examples" / "3-inspectors" / "epu-showcase" / "src" / "benchmarks.rs"


class CliError(Exception):
    def __init__(self, message: str, *, payload: dict[str, Any] | None = None):
        super().__init__(message)
        self.payload = payload or {"ok": False, "error": message}


def parse_bool(value: str) -> bool:
    normalized = value.strip().lower()
    if normalized in {"1", "true", "yes", "on"}:
        return True
    if normalized in {"0", "false", "no", "off"}:
        return False
    raise argparse.ArgumentTypeError(f"invalid boolean value: {value}")


def print_json(payload: Any) -> None:
    sys.stdout.write(json.dumps(payload, indent=2, sort_keys=True))
    sys.stdout.write("\n")


def load_json_text(path_or_dash: str) -> str:
    if path_or_dash == "-":
        return sys.stdin.read()
    return Path(path_or_dash).read_text(encoding="utf-8")


def load_json_value(raw_text: str, *, label: str) -> Any:
    try:
        return json.loads(raw_text)
    except json.JSONDecodeError as exc:
        raise CliError(f"invalid JSON for {label}: {exc}") from exc


def default_session_file() -> Path:
    return DEFAULT_SESSION_FILE


def default_launch_file() -> Path:
    return DEFAULT_LAUNCH_FILE


def launch_file_for_artifacts_dir(artifacts_dir: Path) -> Path:
    return artifacts_dir / "launch.json"


def session_file_for_artifacts_dir(artifacts_dir: Path) -> Path:
    return artifacts_dir / "session.json"


def load_json_path(path: Path, *, label: str) -> Any:
    return load_json_value(path.read_text(encoding="utf-8"), label=label)


def try_load_json_path(path: Path, *, label: str) -> Any | None:
    if not path.is_file():
        return None
    return load_json_path(path, label=label)


def resolve_artifacts_dir(args: argparse.Namespace) -> Path:
    if getattr(args, "artifacts_dir", None):
        return Path(args.artifacts_dir)
    if getattr(args, "session_file", None):
        return Path(args.session_file).parent
    return default_session_file().parent


def resolve_session_file(args: argparse.Namespace) -> Path:
    if getattr(args, "session_file", None):
        return Path(args.session_file)
    if getattr(args, "artifacts_dir", None):
        return session_file_for_artifacts_dir(resolve_artifacts_dir(args))
    return default_session_file()


def resolve_launch_file(args: argparse.Namespace) -> Path:
    if getattr(args, "artifacts_dir", None):
        return launch_file_for_artifacts_dir(resolve_artifacts_dir(args))
    session_file = resolve_session_file(args)
    if session_file == default_session_file():
        return default_launch_file()
    return launch_file_for_artifacts_dir(session_file.parent)


def load_saved_connection_payloads(args: argparse.Namespace) -> tuple[Any | None, Any | None]:
    session_file = resolve_session_file(args)
    launch_file = resolve_launch_file(args)
    session_payload = try_load_json_path(session_file, label=str(session_file))
    launch_payload = try_load_json_path(launch_file, label=str(launch_file))
    return session_payload, launch_payload


def resolve_port(args: argparse.Namespace) -> int:
    if getattr(args, "port", None) is not None:
        return int(args.port)

    session_file = resolve_session_file(args)
    launch_file = resolve_launch_file(args)
    if session_file.is_file():
        payload = load_json_path(session_file, label=str(session_file))
        port = payload.get("port")
        if port is None:
            raise CliError(f"session file does not contain a port: {session_file}")
        return int(port)
    if launch_file.is_file():
        payload = load_json_path(launch_file, label=str(launch_file))
        port = payload.get("port")
        if port is None:
            raise CliError(f"launch file does not contain a port: {launch_file}")
        return int(port)

    raise CliError(
        f"no workbench port provided and no session/launch file found at {session_file.parent}",
        payload={
            "ok": False,
            "error": "missing_connection",
            "message": "Provide --port or launch a session first.",
            "session_file": str(session_file),
            "launch_file": str(launch_file),
            "artifacts_dir": str(session_file.parent),
        },
    )


def base_url(args: argparse.Namespace) -> str:
    return f"http://{args.host}:{resolve_port(args)}{API_PREFIX}"


def api_request(
    args: argparse.Namespace,
    method: str,
    path: str,
    payload: Any | None = None,
    *,
    timeout: float | None = None,
) -> Any:
    url = f"{base_url(args)}{path}"
    body = None
    headers: dict[str, str] = {}
    if payload is not None:
        body = json.dumps(payload).encode("utf-8")
        headers["Content-Type"] = "application/json"

    req = request.Request(url, data=body, headers=headers, method=method)
    try:
        with request.urlopen(req, timeout=timeout or args.timeout) as response:
            raw = response.read().decode("utf-8")
    except error.HTTPError as exc:
        raw = exc.read().decode("utf-8")
        try:
            parsed = json.loads(raw)
        except json.JSONDecodeError:
            parsed = {"ok": False, "error": f"HTTP {exc.code}", "body": raw}
        raise CliError(f"HTTP {exc.code}", payload=parsed) from exc
    except OSError as exc:
        raise CliError(str(exc)) from exc

    try:
        return json.loads(raw)
    except json.JSONDecodeError as exc:
        raise CliError(f"invalid JSON response from {url}: {exc}") from exc


def wait_for_health(host: str, port: int, timeout_seconds: float) -> dict[str, Any]:
    deadline = time.time() + timeout_seconds
    req = request.Request(
        f"http://{host}:{port}{API_PREFIX}/health",
        method="GET",
    )
    last_error = "connection refused"
    while time.time() < deadline:
        try:
            with request.urlopen(req, timeout=2.0) as response:
                return json.loads(response.read().decode("utf-8"))
        except Exception as exc:  # noqa: BLE001
            last_error = str(exc)
            time.sleep(0.5)
    raise CliError(
        f"workbench server did not become healthy within {timeout_seconds:.1f}s",
        payload={
            "ok": False,
            "error": "launch_timeout",
            "host": host,
            "port": port,
            "detail": last_error,
        },
    )


def slugify(value: Any) -> str:
    text = json.dumps(value, sort_keys=True)
    chars: list[str] = []
    for char in text:
        if char.isalnum() or char in {"-", "_"}:
            chars.append(char)
        elif not chars or chars[-1] != "-":
            chars.append("-")
    return "".join(chars).strip("-") or "value"


def build_view_payload(args: argparse.Namespace) -> dict[str, Any]:
    payload: dict[str, Any] = {}
    if args.view_json:
        payload.update(load_json_value(args.view_json, label="view-json"))
    for key in (
        "selected_layer",
        "isolated_layer",
        "locked",
        "show_benchmarks",
        "scene_index",
        "show_ui",
        "show_probe",
        "show_background",
        "camera_angle",
        "camera_elevation",
    ):
        value = getattr(args, key, None)
        if value is not None:
            payload[key] = value
    if args.clear_layer_isolation:
        payload["clear_layer_isolation"] = True
    if not payload:
        raise CliError("set-view requires at least one field")
    return payload


def numeric_sweep_values(args: argparse.Namespace) -> list[Any]:
    if args.values_json:
        values = load_json_value(args.values_json, label="values-json")
        if not isinstance(values, list):
            raise CliError("--values-json must decode to a JSON array")
        return values

    if args.start is None or args.stop is None or args.step is None:
        raise CliError("provide either --values-json or the full --start/--stop/--step range")
    if args.step == 0:
        raise CliError("--step must be non-zero")

    values: list[int] = []
    current = args.start
    if args.step > 0:
        while current <= args.stop:
            values.append(current)
            current += args.step
    else:
        while current >= args.stop:
            values.append(current)
            current += args.step
    return values


def command_health(args: argparse.Namespace) -> dict[str, Any]:
    return api_request(args, "GET", "/health")


def command_session(args: argparse.Namespace) -> dict[str, Any]:
    return api_request(args, "GET", "/session")


def command_status(args: argparse.Namespace) -> dict[str, Any]:
    session_file = resolve_session_file(args)
    launch_file = resolve_launch_file(args)
    artifacts_dir = resolve_artifacts_dir(args)

    payload: dict[str, Any] = {
        "ok": True,
        "host": args.host,
        "artifacts_dir": str(artifacts_dir),
        "session_file": str(session_file),
        "launch_file": str(launch_file),
    }

    try:
        payload["port"] = resolve_port(args)
    except CliError as exc:
        payload["ok"] = False
        payload["connection_error"] = exc.payload
        saved_session, saved_launch = load_saved_connection_payloads(args)
        if saved_session is not None:
            payload["saved_session"] = summarize_session_payload(saved_session)
            payload.update(extract_live_scene_info(saved_session, [], []))
        if saved_launch is not None:
            payload["launch"] = summarize_launch_payload(saved_launch)
        return payload

    try:
        health = command_health(args)
    except CliError as exc:
        payload["connected"] = False
        payload["health_error"] = exc.payload
    else:
        payload["connected"] = True
        payload["health"] = health

    try:
        session = command_session(args)
    except CliError as exc:
        payload["session_error"] = exc.payload
    else:
        payload["session"] = summarize_session_payload(session)
        payload.update(extract_live_scene_info(session, [], []))

    saved_session, saved_launch = load_saved_connection_payloads(args)
    if saved_session is not None:
        payload["saved_session"] = summarize_session_payload(saved_session)
    if saved_launch is not None:
        payload["launch"] = summarize_launch_payload(saved_launch)

    if payload.get("connected") is False and "session" not in payload:
        payload["ok"] = False
    return payload


def command_list_scenes(args: argparse.Namespace) -> dict[str, Any]:
    showcase_names = read_rust_string_array(PRESET_SOURCE, "PRESET_NAMES")
    benchmark_labels = read_rust_string_array(BENCHMARK_SOURCE, "BENCHMARK_NAMES")
    showcase = [
        {"mode": "showcase", "scene_index": index, "name": name}
        for index, name in enumerate(showcase_names)
    ]
    benchmark = [
        {
            "mode": "benchmark",
            "scene_index": index,
            "name": label.removeprefix("Benchmark: ").strip(),
            "label": label,
        }
        for index, label in enumerate(benchmark_labels)
    ]

    session_file = resolve_session_file(args)
    launch_file = resolve_launch_file(args)
    live: dict[str, Any] = {
        "connected": False,
        "session_file": str(session_file),
        "launch_file": str(launch_file),
        "artifacts_dir": str(resolve_artifacts_dir(args)),
    }
    try:
        session_payload = command_session(args)
    except CliError as exc:
        live["error"] = exc.payload
        saved_session, saved_launch = load_saved_connection_payloads(args)
        if saved_session is not None:
            live["saved_session"] = summarize_session_payload(saved_session)
            live.update(extract_live_scene_info(saved_session, showcase, benchmark))
        if saved_launch is not None:
            live["launch"] = summarize_launch_payload(saved_launch)
    else:
        live["connected"] = True
        live["session"] = summarize_session_payload(session_payload)
        live.update(extract_live_scene_info(session_payload, showcase, benchmark))

    return {
        "ok": True,
        "showcase": showcase,
        "benchmark": benchmark,
        "live": live,
        "sources": {
            "showcase": str(PRESET_SOURCE),
            "benchmark": str(BENCHMARK_SOURCE),
        },
    }


def command_get_config(args: argparse.Namespace) -> dict[str, Any]:
    return api_request(args, "GET", "/config")


def command_select_scene(args: argparse.Namespace) -> dict[str, Any]:
    payload = {
        "mode": args.mode,
        "scene_index": args.scene_index,
        "load_into_editor": not args.no_load_into_editor,
        "lock_editor": not args.no_lock_editor,
    }
    return api_request(args, "POST", "/scene", payload)


def command_set_config(args: argparse.Namespace) -> dict[str, Any]:
    if not args.file and not args.config_json:
        raise CliError("set-config requires --file or --config-json")
    raw = args.config_json if args.config_json else load_json_text(args.file)
    payload = load_json_value(raw, label="config")
    return api_request(args, "POST", "/config", payload)


def command_patch_layer(args: argparse.Namespace) -> dict[str, Any]:
    if args.patch_json:
        payload = load_json_value(args.patch_json, label="patch-json")
    elif args.file:
        payload = load_json_value(load_json_text(args.file), label=str(args.file))
    else:
        if not args.field or args.value_json is None:
            raise CliError("patch-layer requires --patch-json, --file, or --field with --value-json")
        payload = {args.field: load_json_value(args.value_json, label="value-json")}
    return api_request(args, "POST", f"/layer/{args.layer}", payload)


def command_set_view(args: argparse.Namespace) -> dict[str, Any]:
    return api_request(args, "POST", "/view", build_view_payload(args))


def command_capture(args: argparse.Namespace) -> dict[str, Any]:
    payload = {"label": args.label} if args.label else None
    return api_request(args, "POST", "/capture", payload, timeout=args.capture_timeout)


def command_export(args: argparse.Namespace) -> dict[str, Any]:
    payload = {
        "label": args.label,
        "rust_const_name": args.rust_const_name,
        "include_json_text": args.include_json_text,
        "include_rust_text": args.include_rust_text,
    }
    return api_request(args, "POST", "/export", payload)


def command_launch(args: argparse.Namespace) -> dict[str, Any]:
    artifacts_dir = Path(args.artifacts_dir).resolve()
    artifacts_dir.mkdir(parents=True, exist_ok=True)

    if args.binary:
        cmd = [str(Path(args.binary).resolve()), str(Path(args.rom).resolve())]
    else:
        cmd = ["cargo", "run", "-p", "nethercore-zx", "--bin", "nethercore-zx"]
        if args.release:
            cmd.append("--release")
        cmd.extend(["--", str(Path(args.rom).resolve())])

    cmd.extend(
        [
            "--epu-workbench-port",
            str(args.port),
            "--epu-workbench-dir",
            str(artifacts_dir),
        ]
    )
    if args.debug:
        cmd.append("--debug")
    if args.fullscreen:
        cmd.append("--fullscreen")
    cmd.extend(args.extra_arg)

    stdout_log = artifacts_dir / "player.stdout.log"
    stderr_log = artifacts_dir / "player.stderr.log"
    creationflags = getattr(subprocess, "CREATE_NEW_PROCESS_GROUP", 0)
    with stdout_log.open("ab") as stdout_handle, stderr_log.open("ab") as stderr_handle:
        proc = subprocess.Popen(
            cmd,
            cwd=str(REPO_ROOT),
            stdout=stdout_handle,
            stderr=stderr_handle,
            creationflags=creationflags,
        )

    health = wait_for_health(args.host, args.port, args.launch_timeout)
    session_args = argparse.Namespace(
        host=args.host,
        port=args.port,
        session_file=args.session_file,
        timeout=args.timeout,
    )
    session = api_request(session_args, "GET", "/session")

    launch_payload = {
        "ok": True,
        "pid": proc.pid,
        "command": cmd,
        "host": args.host,
        "port": args.port,
        "artifacts_dir": str(artifacts_dir),
        "session_file": str(session_file_for_artifacts_dir(artifacts_dir)),
        "launch_file": str(launch_file_for_artifacts_dir(artifacts_dir)),
        "stdout_log": str(stdout_log),
        "stderr_log": str(stderr_log),
        "health": health,
        "session": session,
    }
    (artifacts_dir / "launch.json").write_text(
        json.dumps(launch_payload, indent=2, sort_keys=True),
        encoding="utf-8",
    )
    return launch_payload


def command_sweep_layer(args: argparse.Namespace) -> dict[str, Any]:
    values = numeric_sweep_values(args)
    session = command_session(args)
    artifacts_dir = Path(session.get("artifacts_dir") or resolve_session_file(args).parent)
    sweep_dir = artifacts_dir / "sweeps"
    sweep_dir.mkdir(parents=True, exist_ok=True)

    stamp = time.strftime("%Y%m%d-%H%M%S")
    label_prefix = args.label_prefix or f"layer{args.layer}-{args.field}"
    manifest_path = Path(args.output).resolve() if args.output else sweep_dir / f"{stamp}-{slugify(label_prefix)}.json"

    results: list[dict[str, Any]] = []
    for index, value in enumerate(values):
        label = f"{label_prefix}-{index:03d}-{slugify(value)}"
        patch_payload = {args.field: value}
        step_result: dict[str, Any] = {
            "index": index,
            "value": value,
            "label": label,
            "patch": api_request(args, "POST", f"/layer/{args.layer}", patch_payload),
        }
        if args.capture:
            step_result["capture"] = api_request(
                args,
                "POST",
                "/capture",
                {"label": label},
                timeout=args.capture_timeout,
            )
        if args.export:
            step_result["export"] = api_request(
                args,
                "POST",
                "/export",
                {"label": label},
            )
        results.append(step_result)

    payload = {
        "ok": True,
        "layer": args.layer,
        "field": args.field,
        "values": values,
        "results": results,
        "manifest_path": str(manifest_path),
    }
    manifest_path.write_text(json.dumps(payload, indent=2, sort_keys=True), encoding="utf-8")
    return payload


def read_rust_string_array(path: Path, const_name: str) -> list[str]:
    text = path.read_text(encoding="utf-8")
    anchor = f"pub const {const_name}"
    start = text.find(anchor)
    if start < 0:
        raise CliError(f"could not find {const_name} in {path}")
    equals = text.find("=", start)
    if equals < 0:
        raise CliError(f"could not find assignment for {const_name} in {path}")
    body_start = text.find("[", equals)
    body_end = text.find("];", body_start)
    if body_start < 0 or body_end < 0:
        raise CliError(f"could not parse array body for {const_name} in {path}")
    body = text[body_start + 1 : body_end]
    values = re.findall(r'"([^"\\\\]*(?:\\\\.[^"\\\\]*)*)"', body)
    if not values:
        raise CliError(f"no string values found for {const_name} in {path}")
    return [bytes(value, "utf-8").decode("unicode_escape") for value in values]


def extract_live_scene_info(
    payload: Any,
    showcase: list[dict[str, Any]],
    benchmark: list[dict[str, Any]],
) -> dict[str, Any]:
    if not isinstance(payload, dict):
        return {}

    session_payload = payload.get("session")
    session = session_payload if isinstance(session_payload, dict) else payload
    if not isinstance(session, dict):
        return {}

    mode = session.get("scene_mode")
    scene_index = session.get("scene_index")
    view = session.get("view")
    if not mode and isinstance(view, dict):
        show_benchmarks = view.get("show_benchmarks")
        if show_benchmarks is not None:
            mode = "benchmark" if bool(show_benchmarks) else "showcase"
    if scene_index is None and isinstance(view, dict):
        scene_index = view.get("scene_index")

    info: dict[str, Any] = {
        "scene_mode": mode,
        "scene_index": scene_index,
    }
    if mode in {"showcase", "benchmark"} and isinstance(scene_index, int):
        scenes = benchmark if mode == "benchmark" else showcase
        if 0 <= scene_index < len(scenes):
            info["scene_name"] = scenes[scene_index]["name"]
    return info


def summarize_session_payload(payload: Any) -> dict[str, Any]:
    if not isinstance(payload, dict):
        return {"payload_type": type(payload).__name__}

    session_payload = payload.get("session")
    session = session_payload if isinstance(session_payload, dict) else payload
    if not isinstance(session, dict):
        return {"payload_type": type(session).__name__}

    summary: dict[str, Any] = {}
    for key in ("artifacts_dir", "pid", "port", "updated_at"):
        if key in payload:
            summary[key] = payload[key]
    for key in (
        "game_id",
        "protocol_version",
        "scene_mode",
        "scene_index",
        "show_ui",
        "show_probe",
        "show_background",
    ):
        if key in session:
            summary[key] = session[key]
    view = session.get("view")
    if isinstance(view, dict):
        summary["view"] = {
            key: view.get(key)
            for key in (
                "locked",
                "selected_layer",
                "isolated_layer",
                "show_benchmarks",
                "scene_index",
                "camera_angle",
                "camera_elevation",
            )
        }
    return summary


def summarize_launch_payload(payload: Any) -> dict[str, Any]:
    if not isinstance(payload, dict):
        return {"payload_type": type(payload).__name__}

    summary: dict[str, Any] = {}
    for key in (
        "artifacts_dir",
        "session_file",
        "launch_file",
        "host",
        "port",
        "pid",
        "stdout_log",
        "stderr_log",
    ):
        if key in payload:
            summary[key] = payload[key]
    command = payload.get("command")
    if isinstance(command, list):
        summary["command"] = command
    session = payload.get("session")
    if session is not None:
        summary["session"] = summarize_session_payload(session)
    return summary


def add_connection_args(
    parser: argparse.ArgumentParser,
    *,
    default_port: int | None = None,
    include_artifacts_dir: bool = True,
) -> None:
    parser.add_argument("--host", default=DEFAULT_HOST, help="Workbench host (default: 127.0.0.1)")
    parser.add_argument("--port", type=int, default=default_port, help="Workbench port")
    parser.add_argument(
        "--session-file",
        help=f"Session artifact to read the port from (default: {DEFAULT_SESSION_FILE})",
    )
    if include_artifacts_dir:
        parser.add_argument(
            "--artifacts-dir",
            help="Workbench artifacts directory to resolve session.json/launch.json from",
        )


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)

    launch = subparsers.add_parser("launch", help="Launch a local player with the workbench enabled")
    add_connection_args(launch, default_port=4581, include_artifacts_dir=False)
    launch.add_argument("--rom", required=True, help="ROM to launch")
    launch.add_argument(
        "--artifacts-dir",
        default=str(DEFAULT_SESSION_FILE.parent),
        help="Workbench artifact directory",
    )
    launch.add_argument("--binary", help="Existing nethercore-zx binary to launch instead of cargo")
    launch.add_argument("--release", action="store_true", help="Use `cargo run --release`")
    launch.add_argument("--debug", action="store_true", help="Start the player with --debug")
    launch.add_argument("--fullscreen", action="store_true", help="Start the player fullscreen")
    launch.add_argument("--launch-timeout", type=float, default=60.0, help="Seconds to wait for /health")
    launch.add_argument("--timeout", type=float, default=10.0, help="HTTP request timeout in seconds")
    launch.add_argument(
        "--extra-arg",
        action="append",
        default=[],
        help="Extra raw argument to append to the player command",
    )
    launch.set_defaults(handler=command_launch)

    for name, help_text, handler in (
        ("health", "Check the live workbench server", command_health),
        ("session", "Read the current live workbench snapshot", command_session),
        ("status", "Summarize live health/session plus saved connection artifacts", command_status),
        ("get-config", "Read the current 8-layer EPU config", command_get_config),
    ):
        sub = subparsers.add_parser(name, help=help_text)
        add_connection_args(sub)
        sub.add_argument("--timeout", type=float, default=10.0, help="HTTP request timeout in seconds")
        sub.set_defaults(handler=handler)

    list_scenes = subparsers.add_parser(
        "list-scenes",
        help="List showcase and benchmark scene ids, with optional live-session annotation",
    )
    add_connection_args(list_scenes)
    list_scenes.add_argument("--timeout", type=float, default=10.0, help="HTTP request timeout in seconds")
    list_scenes.set_defaults(handler=command_list_scenes)

    select_scene = subparsers.add_parser("select-scene", help="Select a benchmark or showcase preset")
    add_connection_args(select_scene)
    select_scene.add_argument("--mode", choices=["showcase", "benchmark"], required=True)
    select_scene.add_argument("--scene-index", type=int, required=True)
    select_scene.add_argument("--no-load-into-editor", action="store_true")
    select_scene.add_argument("--no-lock-editor", action="store_true")
    select_scene.add_argument("--timeout", type=float, default=10.0, help="HTTP request timeout in seconds")
    select_scene.set_defaults(handler=command_select_scene)

    set_config = subparsers.add_parser("set-config", help="Replace the full 8-layer workbench config")
    add_connection_args(set_config)
    set_config.add_argument("--file", help="JSON file path or - for stdin")
    set_config.add_argument("--config-json", help="Inline JSON payload")
    set_config.add_argument("--timeout", type=float, default=10.0, help="HTTP request timeout in seconds")
    set_config.set_defaults(handler=command_set_config)

    patch_layer = subparsers.add_parser("patch-layer", help="Patch a single layer")
    add_connection_args(patch_layer)
    patch_layer.add_argument("--layer", type=int, required=True)
    patch_layer.add_argument("--file", help="JSON patch file path or - for stdin")
    patch_layer.add_argument("--patch-json", help="Inline JSON patch payload")
    patch_layer.add_argument("--field", help="Single field name to patch")
    patch_layer.add_argument("--value-json", help="Single JSON value for --field")
    patch_layer.add_argument("--timeout", type=float, default=10.0, help="HTTP request timeout in seconds")
    patch_layer.set_defaults(handler=command_patch_layer)

    set_view = subparsers.add_parser("set-view", help="Update live workbench/editor-facing view state")
    add_connection_args(set_view)
    set_view.add_argument("--view-json", help="Inline JSON payload to merge first")
    set_view.add_argument("--selected-layer", type=int)
    set_view.add_argument("--isolated-layer", type=int)
    set_view.add_argument("--clear-layer-isolation", action="store_true")
    set_view.add_argument("--locked", type=parse_bool)
    set_view.add_argument("--show-benchmarks", type=parse_bool)
    set_view.add_argument("--scene-index", type=int)
    set_view.add_argument("--show-ui", type=parse_bool)
    set_view.add_argument("--show-probe", type=parse_bool)
    set_view.add_argument("--show-background", type=parse_bool)
    set_view.add_argument("--camera-angle", type=float)
    set_view.add_argument("--camera-elevation", type=float)
    set_view.add_argument("--timeout", type=float, default=10.0, help="HTTP request timeout in seconds")
    set_view.set_defaults(handler=command_set_view)

    capture = subparsers.add_parser("capture", help="Capture full frame plus probe/background crops")
    add_connection_args(capture)
    capture.add_argument("--label", help="Capture label prefix")
    capture.add_argument("--timeout", type=float, default=10.0, help="HTTP request timeout in seconds")
    capture.add_argument("--capture-timeout", type=float, default=30.0, help="Seconds to wait for capture completion")
    capture.set_defaults(handler=command_capture)

    export = subparsers.add_parser("export", help="Export the current candidate to JSON and/or Rust")
    add_connection_args(export)
    export.add_argument("--label", help="Export label")
    export.add_argument("--rust-const-name", help="Rust const name for the snippet")
    export.add_argument("--include-json-text", action="store_true")
    export.add_argument("--include-rust-text", action="store_true")
    export.add_argument("--timeout", type=float, default=10.0, help="HTTP request timeout in seconds")
    export.set_defaults(handler=command_export)

    sweep = subparsers.add_parser("sweep-layer", help="Sweep one layer field across a value range")
    add_connection_args(sweep)
    sweep.add_argument("--layer", type=int, required=True)
    sweep.add_argument("--field", required=True)
    sweep.add_argument("--values-json", help="JSON array of values to try")
    sweep.add_argument("--start", type=int)
    sweep.add_argument("--stop", type=int)
    sweep.add_argument("--step", type=int)
    sweep.add_argument("--capture", action="store_true", help="Capture every sweep step")
    sweep.add_argument("--export", action="store_true", help="Export every sweep step")
    sweep.add_argument("--label-prefix", help="Prefix for capture/export labels")
    sweep.add_argument("--output", help="Manifest path for the sweep results")
    sweep.add_argument("--timeout", type=float, default=10.0, help="HTTP request timeout in seconds")
    sweep.add_argument("--capture-timeout", type=float, default=30.0, help="Seconds to wait for each capture")
    sweep.set_defaults(handler=command_sweep_layer)

    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    try:
        result = args.handler(args)
    except CliError as exc:
        print_json(exc.payload)
        return 1
    print_json(result)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
