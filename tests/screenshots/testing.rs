// Copyright © SixtyFPS GmbH <info@slint-ui.com>
// SPDX-License-Identifier: GPL-3.0-only OR LicenseRef-Slint-commercial

use std::rc::Rc;

use i_slint_core::{
    graphics::{
        euclid::{Box2D, Point2D},
        Rgb8Pixel, SharedPixelBuffer,
    },
    renderer::Renderer,
    software_renderer::{LineBufferProvider, MinimalSoftwareWindow},
};

pub struct SwrTestingBackend {
    window: Rc<MinimalSoftwareWindow<0>>,
}

impl i_slint_core::platform::Platform for SwrTestingBackend {
    fn create_window_adapter(&self) -> Rc<dyn i_slint_core::platform::WindowAdapter> {
        self.window.clone()
    }

    fn duration_since_start(&self) -> core::time::Duration {
        core::time::Duration::from_millis(i_slint_core::animations::current_tick().0)
    }
}

pub fn init_swr() -> std::rc::Rc<MinimalSoftwareWindow<0>> {
    let window = MinimalSoftwareWindow::new();

    i_slint_core::platform::set_platform(Box::new(SwrTestingBackend { window: window.clone() }))
        .unwrap();

    window
}

pub fn image_buffer(path: &str) -> SharedPixelBuffer<Rgb8Pixel> {
    let image = image::open(path).expect("Cannot open image.").into_rgb8();

    SharedPixelBuffer::<Rgb8Pixel>::clone_from_slice(image.as_raw(), image.width(), image.height())
}

pub fn screenshot(window: std::rc::Rc<MinimalSoftwareWindow<0>>) -> SharedPixelBuffer<Rgb8Pixel> {
    let size = window.size();
    let width = size.width;
    let height = size.height;

    let mut buffer = SharedPixelBuffer::<Rgb8Pixel>::new(width, height);

    // render to buffer
    window.request_redraw();
    window.draw_if_needed(|renderer| {
        renderer.mark_dirty_region(Box2D::new(
            Point2D::new(0., 0.),
            Point2D::new(width as f32, height as f32),
        ));
        renderer.render(buffer.make_mut_slice(), width as usize);
    });

    buffer
}

struct TestingLineBuffer<'a> {
    buffer: &'a mut [Rgb8Pixel],
}

impl<'a> LineBufferProvider for TestingLineBuffer<'a> {
    type TargetPixel = Rgb8Pixel;

    fn process_line(
        &mut self,
        line: usize,
        range: core::ops::Range<usize>,
        render_fn: impl FnOnce(&mut [Self::TargetPixel]),
    ) {
        let start = line * range.len();
        let end = start + range.len();
        render_fn(&mut self.buffer[(start..end)]);
    }
}

pub fn assert_with_render(path: &str, window: std::rc::Rc<MinimalSoftwareWindow<0>>) {
    assert_eq!(image_buffer(path).as_bytes(), screenshot(window).as_bytes());
}

pub fn assert_with_render_by_line(path: &str, window: std::rc::Rc<MinimalSoftwareWindow<0>>) {
    assert_eq!(image_buffer(path).as_bytes(), screenshot_render_by_line(window).as_bytes());
}

pub fn screenshot_render_by_line(
    window: std::rc::Rc<MinimalSoftwareWindow<0>>,
) -> SharedPixelBuffer<Rgb8Pixel> {
    let size = window.size();
    let width = size.width;
    let height = size.height;

    let mut buffer = SharedPixelBuffer::<Rgb8Pixel>::new(width as u32, height as u32);

    // render to buffer
    window.request_redraw();
    window.draw_if_needed(|renderer| {
        renderer.mark_dirty_region(Box2D::new(
            Point2D::new(0., 0.),
            Point2D::new(width as f32, height as f32),
        ));
        renderer.render_by_line(TestingLineBuffer { buffer: buffer.make_mut_slice() });
    });

    buffer
}

pub fn save_screenshot(path: &str, window: std::rc::Rc<MinimalSoftwareWindow<0>>) {
    let buffer = screenshot(window.clone());
    image::save_buffer(
        path,
        buffer.as_bytes(),
        window.size().width,
        window.size().height,
        image::ColorType::Rgb8,
    )
    .unwrap();
}