//! Local HTTP control surface for the live EPU workbench.

use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;

use anyhow::{Context, Result};
use chrono::Local;
use serde::de::DeserializeOwned;
use serde_json::json;

use crate::capture::{CaptureSupport, read_render_target_pixels};
use crate::console::Console;
use crate::workbench::{
    EPU_WORKBENCH_PROTOCOL_VERSION, EpuWorkbenchCaptureResult, EpuWorkbenchConfig,
    EpuWorkbenchCrop, EpuWorkbenchExportOptions, EpuWorkbenchExportResult, EpuWorkbenchLayerPatch,
    EpuWorkbenchMetadata, EpuWorkbenchSceneMode, EpuWorkbenchSceneSelection, EpuWorkbenchSnapshot,
    EpuWorkbenchViewState,
};

use super::StandaloneApp;
use super::types::{RomLoader, StandaloneGraphicsSupport};

const LOCALHOST: &str = "127.0.0.1";

pub(super) struct WorkbenchBridge {
    request_rx: mpsc::Receiver<WorkbenchEnvelope>,
    artifacts_dir: PathBuf,
    pending_capture: Option<PendingCapture>,
    port: u16,
}

struct PendingCapture {
    label: String,
    response_tx: mpsc::Sender<HttpResponse>,
}

struct WorkbenchEnvelope {
    request: HttpRequest,
    response_tx: mpsc::Sender<HttpResponse>,
}

struct HttpRequest {
    method: String,
    path: String,
    body: Vec<u8>,
}

struct HttpResponse {
    status: u16,
    body: Vec<u8>,
}

impl HttpResponse {
    fn json(value: serde_json::Value) -> Self {
        Self {
            status: 200,
            body: serde_json::to_vec(&value).unwrap_or_else(|_| br#"{"ok":false}"#.to_vec()),
        }
    }

    fn error(status: u16, message: impl Into<String>) -> Self {
        Self::json(json!({
            "ok": false,
            "error": message.into(),
        }))
        .with_status(status)
    }

    fn with_status(mut self, status: u16) -> Self {
        self.status = status;
        self
    }
}

impl WorkbenchBridge {
    pub(super) fn start(port: u16, artifacts_dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&artifacts_dir).with_context(|| {
            format!("failed to create workbench dir {}", artifacts_dir.display())
        })?;

        let listener = TcpListener::bind((LOCALHOST, port))
            .with_context(|| format!("failed to bind workbench server on {LOCALHOST}:{port}"))?;
        let actual_port = listener
            .local_addr()
            .context("failed to query workbench bind address")?
            .port();

        let (request_tx, request_rx) = mpsc::channel::<WorkbenchEnvelope>();
        thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(stream) = stream else {
                    continue;
                };
                let request_tx = request_tx.clone();
                thread::spawn(move || {
                    let _ = handle_stream(stream, request_tx);
                });
            }
        });

        Ok(Self {
            request_rx,
            artifacts_dir,
            pending_capture: None,
            port: actual_port,
        })
    }

    pub(super) fn port(&self) -> u16 {
        self.port
    }
}

fn handle_stream(stream: TcpStream, request_tx: mpsc::Sender<WorkbenchEnvelope>) -> Result<()> {
    let request = read_http_request(stream.try_clone().context("clone request stream")?)?;
    let (response_tx, response_rx) = mpsc::channel();
    request_tx
        .send(WorkbenchEnvelope {
            request,
            response_tx,
        })
        .context("dispatch workbench request")?;
    let response = response_rx
        .recv()
        .context("receive workbench response from main thread")?;
    write_http_response(stream, response)?;
    Ok(())
}

fn read_http_request(stream: TcpStream) -> Result<HttpRequest> {
    let mut reader = BufReader::new(stream);
    let mut request_line = String::new();
    reader
        .read_line(&mut request_line)
        .context("read request line")?;
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("").to_string();
    let path = parts.next().unwrap_or("").to_string();
    if method.is_empty() || path.is_empty() {
        anyhow::bail!("invalid HTTP request line");
    }

    let mut content_length = 0usize;
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).context("read header line")?;
        if line == "\r\n" || line == "\n" || line.is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':')
            && name.eq_ignore_ascii_case("content-length")
        {
            content_length = value.trim().parse().unwrap_or(0);
        }
    }

    let mut body = vec![0u8; content_length];
    if content_length > 0 {
        reader.read_exact(&mut body).context("read request body")?;
    }

    Ok(HttpRequest { method, path, body })
}

fn write_http_response(mut stream: TcpStream, response: HttpResponse) -> Result<()> {
    let status_text = match response.status {
        200 => "OK",
        202 => "Accepted",
        400 => "Bad Request",
        404 => "Not Found",
        409 => "Conflict",
        500 => "Internal Server Error",
        503 => "Service Unavailable",
        _ => "OK",
    };
    let headers = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        response.status,
        status_text,
        response.body.len()
    );
    stream.write_all(headers.as_bytes())?;
    stream.write_all(&response.body)?;
    stream.flush()?;
    Ok(())
}

impl<C, L> StandaloneApp<C, L>
where
    C: Console + Clone,
    C::Graphics: StandaloneGraphicsSupport,
    L: RomLoader<Console = C>,
{
    pub(super) fn maybe_start_workbench(&mut self) -> Result<()> {
        if self.workbench.is_some() {
            return Ok(());
        }
        let Some(port) = self.config.epu_workbench_port else {
            return Ok(());
        };

        let dir = self
            .config
            .epu_workbench_dir
            .clone()
            .unwrap_or_else(|| PathBuf::from("tmp").join("epu-workbench"));
        self.workbench = Some(WorkbenchBridge::start(port, dir)?);
        if let Err(error) = self.workbench_snapshot() {
            tracing::warn!("failed to persist initial EPU workbench session state: {error:#}");
        }
        Ok(())
    }

    pub(super) fn process_workbench_requests(&mut self) {
        loop {
            let envelope = {
                let Some(workbench) = &self.workbench else {
                    return;
                };
                match workbench.request_rx.try_recv() {
                    Ok(env) => env,
                    Err(mpsc::TryRecvError::Empty) | Err(mpsc::TryRecvError::Disconnected) => {
                        return;
                    }
                }
            };

            let response = match self
                .handle_workbench_request(envelope.request, envelope.response_tx.clone())
            {
                Ok(Some(response)) => response,
                Ok(None) => continue,
                Err(error) => HttpResponse::error(500, error.to_string()),
            };
            let _ = envelope.response_tx.send(response);
        }
    }

    pub(super) fn maybe_complete_workbench_capture(&mut self) {
        let Some(workbench) = &mut self.workbench else {
            return;
        };
        let Some(pending) = workbench.pending_capture.take() else {
            return;
        };
        let Some(runner) = &self.runner else {
            let _ = pending
                .response_tx
                .send(HttpResponse::error(503, "runner not available"));
            return;
        };

        let (width, height) = runner.graphics().render_target_dimensions();
        let pixels = read_render_target_pixels(
            runner.graphics().device(),
            runner.graphics().queue(),
            runner.graphics().render_target_texture(),
            width,
            height,
        );

        let crops = runner
            .session()
            .map(|session| {
                session
                    .runtime
                    .console()
                    .epu_workbench_capture_crops(width, height)
            })
            .unwrap_or_else(|| default_capture_crops(width, height));

        let capture_result = save_capture_bundle(
            &workbench.artifacts_dir,
            &pending.label,
            &pixels,
            width,
            height,
            crops,
        );

        let response = match capture_result {
            Ok(result) => HttpResponse::json(json!({
                "ok": true,
                "capture": result,
            })),
            Err(error) => HttpResponse::error(500, error.to_string()),
        };
        let _ = pending.response_tx.send(response);
    }

    fn handle_workbench_request(
        &mut self,
        request: HttpRequest,
        response_tx: mpsc::Sender<HttpResponse>,
    ) -> Result<Option<HttpResponse>> {
        match (request.method.as_str(), request.path.as_str()) {
            ("GET", "/api/epu-workbench/health") => Ok(Some(HttpResponse::json(json!({
                "ok": true,
                "protocol_version": EPU_WORKBENCH_PROTOCOL_VERSION,
                "pid": std::process::id(),
                "port": self.workbench.as_ref().map(|w| w.port()),
            })))),
            ("GET", "/api/epu-workbench/session") => {
                let snapshot = self.workbench_snapshot()?;
                Ok(Some(HttpResponse::json(json!({
                    "ok": true,
                    "session": snapshot,
                    "artifacts_dir": self.workbench.as_ref().map(|w| w.artifacts_dir.display().to_string()),
                    "port": self.workbench.as_ref().map(|w| w.port()),
                    "pid": std::process::id(),
                }))))
            }
            ("GET", "/api/epu-workbench/config") => {
                let config = self.current_workbench_config()?;
                Ok(Some(HttpResponse::json(json!({
                    "ok": true,
                    "config": config,
                }))))
            }
            ("POST", "/api/epu-workbench/scene") => {
                let body: EpuWorkbenchSceneSelection = parse_json(&request.body)?;
                self.apply_scene_selection(&body)?;
                let snapshot = self.workbench_snapshot()?;
                Ok(Some(HttpResponse::json(json!({
                    "ok": true,
                    "session": snapshot,
                }))))
            }
            ("POST", "/api/epu-workbench/config") => {
                let body: EpuWorkbenchConfig = parse_json(&request.body)?;
                self.set_workbench_config(&body)?;
                let snapshot = self.workbench_snapshot()?;
                Ok(Some(HttpResponse::json(json!({
                    "ok": true,
                    "session": snapshot,
                }))))
            }
            ("POST", path) if path.starts_with("/api/epu-workbench/layer/") => {
                let layer_index = path
                    .trim_start_matches("/api/epu-workbench/layer/")
                    .parse::<usize>()
                    .context("invalid layer index")?;
                let body: EpuWorkbenchLayerPatch = parse_json(&request.body)?;
                self.patch_workbench_layer(layer_index, &body)?;
                let snapshot = self.workbench_snapshot()?;
                Ok(Some(HttpResponse::json(json!({
                    "ok": true,
                    "session": snapshot,
                }))))
            }
            ("POST", "/api/epu-workbench/view") => {
                let body: EpuWorkbenchViewState = parse_json(&request.body)?;
                self.set_workbench_view(&body)?;
                let snapshot = self.workbench_snapshot()?;
                Ok(Some(HttpResponse::json(json!({
                    "ok": true,
                    "session": snapshot,
                }))))
            }
            ("POST", "/api/epu-workbench/capture") => {
                let payload = if request.body.is_empty() {
                    json!({})
                } else {
                    serde_json::from_slice::<serde_json::Value>(&request.body)
                        .context("parse capture request")?
                };
                let label = payload
                    .get("label")
                    .and_then(|value| value.as_str())
                    .unwrap_or("capture")
                    .to_string();
                let workbench = self.workbench.as_mut().context("workbench unavailable")?;
                if workbench.pending_capture.is_some() {
                    return Ok(Some(HttpResponse::error(409, "capture already pending")));
                }
                workbench.pending_capture = Some(PendingCapture { label, response_tx });
                self.needs_redraw = true;
                Ok(None)
            }
            ("POST", "/api/epu-workbench/export") => {
                let body: EpuWorkbenchExportOptions = if request.body.is_empty() {
                    EpuWorkbenchExportOptions::default()
                } else {
                    parse_json(&request.body)?
                };
                let export = self.export_workbench(&body)?;
                Ok(Some(HttpResponse::json(json!({
                    "ok": true,
                    "export": export,
                }))))
            }
            _ => Ok(Some(HttpResponse::error(
                404,
                format!("unknown endpoint {}", request.path),
            ))),
        }
    }

    fn workbench_snapshot(&mut self) -> Result<EpuWorkbenchSnapshot> {
        self.sync_console_debug_state();
        let config = self.current_workbench_config()?;
        let view = self
            .runner
            .as_ref()
            .and_then(|runner| {
                runner
                    .session()
                    .and_then(|session| session.runtime.console().epu_workbench_view())
            })
            .unwrap_or_default();
        let metadata = self
            .runner
            .as_ref()
            .and_then(|runner| {
                runner
                    .session()
                    .map(|session| session.runtime.console().epu_workbench_metadata())
            })
            .unwrap_or_else(EpuWorkbenchMetadata::default);

        let scene_mode =
            self.read_debug_value_bool("scene/show_benchmarks")?
                .map(|show_benchmarks| {
                    if show_benchmarks {
                        EpuWorkbenchSceneMode::Benchmark
                    } else {
                        EpuWorkbenchSceneMode::Showcase
                    }
                });

        let snapshot = EpuWorkbenchSnapshot {
            protocol_version: EPU_WORKBENCH_PROTOCOL_VERSION,
            game_id: self
                .loaded_rom
                .as_ref()
                .map(|rom| rom.game_id.clone())
                .unwrap_or_else(|| "unknown".to_string()),
            scene_mode,
            scene_index: self.read_debug_value_i32("scene/index")?,
            show_ui: self.read_debug_value_bool("scene/show_ui")?,
            show_probe: self.read_debug_value_bool("scene/show_probe")?,
            show_background: self.read_debug_value_bool("scene/show_background")?,
            config,
            view,
            metadata,
        };
        self.persist_workbench_session(&snapshot)?;
        Ok(snapshot)
    }

    fn current_workbench_config(&self) -> Result<EpuWorkbenchConfig> {
        let runner = self.runner.as_ref().context("runner not initialized")?;
        let Some(session) = runner.session() else {
            anyhow::bail!("session not initialized");
        };
        if !session.runtime.console().has_epu_workbench() {
            anyhow::bail!("loaded ROM console does not support the EPU workbench")
        }
        session
            .runtime
            .console()
            .epu_workbench_config()
            .context("console did not return a workbench config")
    }

    fn set_workbench_config(&mut self, config: &EpuWorkbenchConfig) -> Result<()> {
        let runner = self.runner.as_mut().context("runner not initialized")?;
        runner
            .console_mut()
            .context("console unavailable")?
            .epu_workbench_set_config(config)?;
        self.refresh_render_state()?;
        Ok(())
    }

    fn patch_workbench_layer(
        &mut self,
        layer_index: usize,
        patch: &EpuWorkbenchLayerPatch,
    ) -> Result<()> {
        let runner = self.runner.as_mut().context("runner not initialized")?;
        runner
            .console_mut()
            .context("console unavailable")?
            .epu_workbench_patch_layer(layer_index, patch)?;
        self.refresh_render_state()?;
        Ok(())
    }

    fn set_workbench_view(&mut self, view: &EpuWorkbenchViewState) -> Result<()> {
        if let Some(show_benchmarks) = view.show_benchmarks {
            self.write_debug_value("scene/show_benchmarks", json!(show_benchmarks))?;
        }
        if let Some(scene_index) = view.scene_index {
            self.write_debug_value("scene/index", json!(scene_index))?;
        }
        if let Some(show_ui) = view.show_ui {
            self.write_debug_value("scene/show_ui", json!(show_ui))?;
        }
        if let Some(show_probe) = view.show_probe {
            self.write_debug_value("scene/show_probe", json!(show_probe))?;
        }
        if let Some(show_background) = view.show_background {
            self.write_debug_value("scene/show_background", json!(show_background))?;
        }

        let runner = self.runner.as_mut().context("runner not initialized")?;
        runner
            .console_mut()
            .context("console unavailable")?
            .epu_workbench_set_view(view)?;
        self.refresh_render_state()?;
        Ok(())
    }

    fn export_workbench(
        &mut self,
        options: &EpuWorkbenchExportOptions,
    ) -> Result<EpuWorkbenchExportResult> {
        let runner = self.runner.as_ref().context("runner not initialized")?;
        let session = runner.session().context("session not initialized")?;
        let mut export = session.runtime.console().epu_workbench_export(options)?;
        let artifacts_dir = self
            .workbench
            .as_ref()
            .context("workbench unavailable")?
            .artifacts_dir
            .join("exports");
        fs::create_dir_all(&artifacts_dir)?;
        let stamp = Local::now().format("%Y%m%d-%H%M%S");
        let label = sanitize_label(options.label.as_deref().unwrap_or("candidate"));
        if let Some(json_text) = &export.json_text {
            let path = artifacts_dir.join(format!("{stamp}-{label}.json"));
            fs::write(&path, json_text)?;
            export.json_path = Some(path.display().to_string());
        }
        if let Some(rust_text) = &export.rust_text {
            let path = artifacts_dir.join(format!("{stamp}-{label}.rs"));
            fs::write(&path, rust_text)?;
            export.rust_path = Some(path.display().to_string());
        }
        Ok(export)
    }

    fn persist_workbench_session(&self, snapshot: &EpuWorkbenchSnapshot) -> Result<()> {
        let Some(workbench) = &self.workbench else {
            return Ok(());
        };
        let payload = json!({
            "updated_at": Local::now().to_rfc3339(),
            "pid": std::process::id(),
            "port": workbench.port(),
            "artifacts_dir": workbench.artifacts_dir.display().to_string(),
            "session": snapshot,
        });
        let path = workbench.artifacts_dir.join("session.json");
        let bytes = serde_json::to_vec_pretty(&payload).context("serialize workbench session")?;
        fs::write(&path, bytes)
            .with_context(|| format!("failed to write workbench session {}", path.display()))
    }

    fn apply_scene_selection(&mut self, selection: &EpuWorkbenchSceneSelection) -> Result<()> {
        self.write_debug_value(
            "scene/show_benchmarks",
            json!(matches!(selection.mode, EpuWorkbenchSceneMode::Benchmark)),
        )?;
        self.write_debug_value("scene/index", json!(selection.scene_index))?;
        self.refresh_render_state()?;
        self.sync_console_debug_state();

        if selection.load_into_editor {
            let runner = self.runner.as_mut().context("runner not initialized")?;
            if let Some(console) = runner.console_mut() {
                console.epu_workbench_load_snapshot_env(0)?;
                console.epu_workbench_set_view(&EpuWorkbenchViewState {
                    selected_layer: Some(0),
                    clear_layer_isolation: Some(true),
                    locked: Some(selection.lock_editor),
                    ..Default::default()
                })?;
            }
        }

        self.refresh_render_state()?;
        Ok(())
    }

    fn refresh_render_state(&mut self) -> Result<()> {
        let runner = self.runner.as_mut().context("runner not initialized")?;
        let session = runner.session_mut().context("session not initialized")?;
        if let Some(game) = session.runtime.game_mut() {
            C::clear_frame_state(game.console_state_mut());
        }
        session
            .runtime
            .render()
            .context("refreshing live render state failed")?;
        self.execute_draw_commands();
        self.last_sim_rendered = true;
        self.needs_redraw = true;
        Ok(())
    }

    fn sync_console_debug_state(&mut self) {
        let Some(runner) = &mut self.runner else {
            return;
        };
        let Some(session) = runner.session_mut() else {
            return;
        };
        let (console, state_opt) = session.runtime.console_and_state_mut();
        if let Some(state) = state_opt {
            console.sync_debug_ui_state(state);
        }
    }

    fn read_debug_value_i32(&self, full_path: &str) -> Result<Option<i32>> {
        let Some(runner) = &self.runner else {
            return Ok(None);
        };
        let Some(session) = runner.session() else {
            return Ok(None);
        };
        let Some(game) = session.runtime.game() else {
            return Ok(None);
        };
        let registry = game.store().data().debug_registry.clone();
        let Some(value) = registry
            .values
            .iter()
            .find(|value| value.full_path == full_path)
        else {
            return Ok(None);
        };
        let Some(memory) = game.store().data().game.memory else {
            return Ok(None);
        };
        let data = memory.data(game.store());
        let ptr = value.wasm_ptr as usize;
        let size = value.value_type.byte_size();
        if ptr + size > data.len() {
            return Ok(None);
        }
        let debug_value = registry.read_value_from_slice(&data[ptr..ptr + size], value.value_type);
        Ok(match debug_value {
            crate::debug::types::DebugValue::I32(value) => Some(value),
            _ => None,
        })
    }

    fn read_debug_value_bool(&self, full_path: &str) -> Result<Option<bool>> {
        let Some(runner) = &self.runner else {
            return Ok(None);
        };
        let Some(session) = runner.session() else {
            return Ok(None);
        };
        let Some(game) = session.runtime.game() else {
            return Ok(None);
        };
        let registry = game.store().data().debug_registry.clone();
        let Some(value) = registry
            .values
            .iter()
            .find(|value| value.full_path == full_path)
        else {
            return Ok(None);
        };
        let Some(memory) = game.store().data().game.memory else {
            return Ok(None);
        };
        let data = memory.data(game.store());
        let ptr = value.wasm_ptr as usize;
        let size = value.value_type.byte_size();
        if ptr + size > data.len() {
            return Ok(None);
        }
        let debug_value = registry.read_value_from_slice(&data[ptr..ptr + size], value.value_type);
        Ok(match debug_value {
            crate::debug::types::DebugValue::Bool(value) => Some(value),
            _ => None,
        })
    }

    fn write_debug_value(&mut self, full_path: &str, value: serde_json::Value) -> Result<()> {
        let runner = self.runner.as_mut().context("runner not initialized")?;
        let session = runner.session_mut().context("session not initialized")?;
        let game = session.runtime.game_mut().context("game not initialized")?;
        let registry = game.store().data().debug_registry.clone();
        let reg_value = registry
            .values
            .iter()
            .find(|item| item.full_path == full_path)
            .cloned()
            .with_context(|| format!("debug value not found: {full_path}"))?;

        let memory = game
            .store()
            .data()
            .game
            .memory
            .context("debug memory unavailable")?;
        let debug_value = match reg_value.value_type {
            crate::debug::types::ValueType::Bool => crate::debug::types::DebugValue::Bool(
                value.as_bool().context("expected bool debug value")?,
            ),
            crate::debug::types::ValueType::I32 => crate::debug::types::DebugValue::I32(
                value.as_i64().context("expected i32 debug value")? as i32,
            ),
            _ => anyhow::bail!("unsupported debug write type for {full_path}"),
        };

        let store = game.store_mut();
        let data = memory.data_mut(store);
        let ptr = reg_value.wasm_ptr as usize;
        let size = reg_value.value_type.byte_size();
        if ptr + size > data.len() {
            anyhow::bail!("debug write out of bounds: {full_path}");
        }
        registry.write_value_to_slice(&mut data[ptr..ptr + size], &debug_value);
        if game.has_debug_change_callback() {
            game.call_on_debug_change();
        }
        Ok(())
    }
}

fn parse_json<T: DeserializeOwned>(body: &[u8]) -> Result<T> {
    serde_json::from_slice(body).context("invalid JSON body")
}

fn sanitize_label(label: &str) -> String {
    let mut output = String::with_capacity(label.len());
    for ch in label.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            output.push(ch);
        } else if !output.ends_with('-') {
            output.push('-');
        }
    }
    output.trim_matches('-').to_string()
}

fn save_capture_bundle(
    artifacts_dir: &Path,
    label: &str,
    pixels: &[u8],
    width: u32,
    height: u32,
    crops: crate::workbench::EpuWorkbenchCaptureCrops,
) -> Result<EpuWorkbenchCaptureResult> {
    fs::create_dir_all(artifacts_dir)?;
    let captures_dir = artifacts_dir.join("captures");
    fs::create_dir_all(&captures_dir)?;
    let stamp = Local::now().format("%Y%m%d-%H%M%S");
    let label = sanitize_label(label);
    let base = captures_dir.join(format!("{stamp}-{label}"));

    let frame_path = base.with_extension("png");
    save_rgba_png(&frame_path, pixels, width, height)?;

    let background_pixels = crop_rgba(pixels, width, height, crops.background)?;
    let background_path = captures_dir.join(format!("{stamp}-{label}-background.png"));
    save_rgba_png(
        &background_path,
        &background_pixels,
        crops.background.width,
        crops.background.height,
    )?;

    let probe_pixels = crop_rgba(pixels, width, height, crops.probe)?;
    let probe_path = captures_dir.join(format!("{stamp}-{label}-probe.png"));
    save_rgba_png(
        &probe_path,
        &probe_pixels,
        crops.probe.width,
        crops.probe.height,
    )?;

    Ok(EpuWorkbenchCaptureResult {
        frame_path: frame_path.display().to_string(),
        background_crop_path: background_path.display().to_string(),
        probe_crop_path: probe_path.display().to_string(),
        width,
        height,
        background_crop: crops.background,
        probe_crop: crops.probe,
    })
}

fn crop_rgba(pixels: &[u8], width: u32, height: u32, crop: EpuWorkbenchCrop) -> Result<Vec<u8>> {
    if crop.x + crop.width > width || crop.y + crop.height > height {
        anyhow::bail!("crop out of bounds");
    }
    let mut output = Vec::with_capacity((crop.width * crop.height * 4) as usize);
    for row in crop.y..(crop.y + crop.height) {
        let start = ((row * width + crop.x) * 4) as usize;
        let end = start + (crop.width * 4) as usize;
        output.extend_from_slice(&pixels[start..end]);
    }
    Ok(output)
}

fn save_rgba_png(path: &Path, pixels: &[u8], width: u32, height: u32) -> Result<()> {
    let image = image::RgbaImage::from_raw(width, height, pixels.to_vec())
        .context("failed to build RGBA image")?;
    image
        .save(path)
        .with_context(|| format!("failed to save {}", path.display()))
}

fn default_capture_crops(width: u32, height: u32) -> crate::workbench::EpuWorkbenchCaptureCrops {
    crate::workbench::EpuWorkbenchCaptureCrops {
        background: EpuWorkbenchCrop {
            x: width / 16,
            y: height / 16,
            width: width / 2,
            height: height / 2,
        },
        probe: EpuWorkbenchCrop {
            x: width / 3,
            y: height / 6,
            width: width / 3,
            height: height * 2 / 3,
        },
    }
}

