use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use crabgrab::{
    feature::{bitmap::VideoFrameBitmap, screenshot::take_screenshot},
    prelude::*,
};
use png::{BitDepth, ColorType, Encoder};
use serde::Serialize;
use std::io::Cursor;

#[derive(Debug, Serialize)]
pub struct MonitorCapture {
    pub position: (i32, i32),
    pub data_url: String,
}

pub async fn capture_monitors(
    window: &tauri::WebviewWindow,
) -> Result<Vec<MonitorCapture>, String> {
    let token = CaptureStream::test_access(false)
        .or(CaptureStream::request_access(false).await)
        .ok_or_else(|| "Screen Recording permission was not granted".to_string())?;

    let mut monitors = window
        .available_monitors()
        .map_err(|error| error.to_string())?;
    monitors.sort_by_key(|monitor| (monitor.position().x, monitor.position().y));

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

    let mut captures = Vec::with_capacity(monitors.len());
    for (monitor, display) in monitors.into_iter().zip(displays) {
        let frame = take_screenshot(
            token,
            CaptureConfig::with_display(display, CapturePixelFormat::Bgra8888),
        )
        .await
        .map_err(|error| error.to_string())?;
        let bitmap = frame.get_bitmap().map_err(|error| error.to_string())?;
        let png = encode_png(bitmap)?;
        let position = monitor.position();

        captures.push(MonitorCapture {
            position: (position.x, position.y),
            data_url: format!("data:image/png;base64,{}", BASE64.encode(png)),
        });
    }

    Ok(captures)
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
