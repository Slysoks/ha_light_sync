use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
#[cfg(target_os = "windows")]
use crabgrab::platform::windows::WindowsCaptureConfigExt;
use crabgrab::{feature::bitmap::VideoFrameBitmap, prelude::*};
use png::{BitDepth, ColorType, Encoder};
use serde::Serialize;
use std::{
    io::Cursor,
    sync::{Arc, Mutex},
};
use tauri::State;

#[derive(Debug, Serialize)]
pub struct MonitorCapture {
    pub position: (i32, i32),
    pub data_url: String,
}

#[derive(Default)]
pub struct CaptureState {
    active: Mutex<Option<ActiveCapture>>,
}

struct ActiveCapture {
    monitors: Vec<(i32, i32, u32, u32)>,
    _streams: Vec<CaptureStream>,
    frames: Vec<Arc<Mutex<Option<String>>>>,
}

pub async fn capture_monitors(
    window: &tauri::WebviewWindow,
    state: State<'_, CaptureState>,
) -> Result<Vec<MonitorCapture>, String> {
    let mut monitors = window
        .available_monitors()
        .map_err(|error| error.to_string())?;
    monitors.sort_by_key(|monitor| (monitor.position().x, monitor.position().y));
    let monitor_keys = monitors
        .iter()
        .map(|monitor| {
            let position = monitor.position();
            let size = monitor.size();
            (position.x, position.y, size.width, size.height)
        })
        .collect::<Vec<_>>();

    if let Some(captures) = read_cached_captures(&state, &monitor_keys) {
        return Ok(captures);
    }

    let token = {
        #[cfg(target_os = "windows")]
        {
            CaptureStream::test_access(true).or(CaptureStream::request_access(true).await)
        }
        #[cfg(not(target_os = "windows"))]
        {
            CaptureStream::test_access(false).or(CaptureStream::request_access(false).await)
        }
    }
    .ok_or_else(|| "Screen Recording permission was not granted".to_string())?;

    let content = CapturableContent::new(CapturableContentFilter::DISPLAYS)
        .await
        .map_err(|error| error.to_string())?;
    let mut displays = content.displays().collect::<Vec<_>>();
    displays.sort_by(|left, right| {
        left.rect()
            .origin
            .x
            .total_cmp(&right.rect().origin.x)
            .then(left.rect().origin.y.total_cmp(&right.rect().origin.y))
    });

    if monitors.len() != displays.len() {
        return Err(format!(
            "The window API found {} monitors, but CrabGrab found {} displays",
            monitors.len(),
            displays.len()
        ));
    }

    let mut streams = Vec::with_capacity(monitors.len());
    let mut frames = Vec::with_capacity(monitors.len());
    for (_monitor, display) in monitors.into_iter().zip(displays) {
        let config = CaptureConfig::with_display(display, CapturePixelFormat::Bgra8888);
        #[cfg(target_os = "windows")]
        let config = config.with_borderless(true);

        let frame_data = Arc::new(Mutex::new(None));
        let callback_data = Arc::clone(&frame_data);
        let stream = CaptureStream::new(token, config, move |event| match event {
            Ok(StreamEvent::Video(frame)) => {
                match frame
                    .get_bitmap()
                    .map_err(|error| error.to_string())
                    .and_then(encode_png)
                {
                    Ok(png) => match callback_data.lock() {
                        Ok(mut data) => {
                            *data = Some(format!("data:image/png;base64,{}", BASE64.encode(png)));
                        }
                        Err(_) => eprintln!("Unable to update monitor capture cache"),
                    },
                    Err(error) => eprintln!("Unable to encode monitor capture: {error}"),
                }
            }
            Ok(_) => {}
            Err(error) => eprintln!("Monitor capture stream failed: {error}"),
        })
        .map_err(|error| error.to_string())?;

        streams.push(stream);
        frames.push(frame_data);
    }

    let mut active = state
        .active
        .lock()
        .map_err(|_| "Capture state is unavailable".to_string())?;
    let new_active = ActiveCapture {
        monitors: monitor_keys,
        _streams: streams,
        frames,
    };
    let captures = read_active_captures(&new_active);
    *active = Some(new_active);

    Ok(captures)
}

fn read_cached_captures(
    state: &State<'_, CaptureState>,
    monitor_keys: &[(i32, i32, u32, u32)],
) -> Option<Vec<MonitorCapture>> {
    let active = state.active.lock().ok()?;
    let active = active.as_ref()?;
    (active.monitors == monitor_keys).then(|| read_active_captures(active))
}

fn read_active_captures(active: &ActiveCapture) -> Vec<MonitorCapture> {
    active
        .monitors
        .iter()
        .zip(&active.frames)
        .filter_map(|(monitor, frame)| {
            frame
                .lock()
                .ok()
                .and_then(|data| data.clone())
                .map(|data_url| MonitorCapture {
                    position: (monitor.0, monitor.1),
                    data_url,
                })
        })
        .collect()
}

fn encode_png(bitmap: crabgrab::feature::bitmap::BoxedSliceFrameBitmap) -> Result<Vec<u8>, String> {
    let bitmap = match bitmap {
        crabgrab::feature::bitmap::FrameBitmap::BgraUnorm8x4(bitmap) => bitmap,
        _ => return Err("CrabGrab returned an unsupported pixel format".to_string()),
    };

    let mut rgba = Vec::with_capacity(bitmap.data.len() * 4);
    for pixel in bitmap.data.as_ref() {
        rgba.extend_from_slice(&[pixel[2], pixel[1], pixel[0], pixel[3]]);
    }

    let mut output = Vec::new();
    {
        let mut encoder = Encoder::new(
            Cursor::new(&mut output),
            bitmap.width as u32,
            bitmap.height as u32,
        );
        encoder.set_color(ColorType::Rgba);
        encoder.set_depth(BitDepth::Eight);
        encoder
            .write_header()
            .map_err(|error| error.to_string())?
            .write_image_data(&rgba)
            .map_err(|error| error.to_string())?;
    }

    Ok(output)
}
