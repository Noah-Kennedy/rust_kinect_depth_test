extern crate freenectrs;
#[macro_use]
extern crate glium;
extern crate obj;
extern crate genmesh;
extern crate image;

mod glwinhelp;

use glwinhelp::imgwin;

use std::time::Duration;
use std::io::Result;
use freenectrs::freenect;

#[inline]
fn convert_raw_kinect_data_to_meters(raw_depth: f32) -> f32 {
    return 1.0 / (raw_depth * -0.0030711016 + 3.3309495161);
}

#[inline]
fn spectrum_curve(input: f32) -> f32 {
    return f32::min(f32::max(2.0 - input.abs(), 0.0), 1.0) * 255.0;
}

#[inline]
fn convert_depth_to_rgb(distance: f32) -> (u8, u8, u8) {
    if distance < 0.0 {
        return (0, 0, 0);
    }

    let decimal: f32 = distance - ((distance as u8) as f32);

    let spectrum: f32 = 6.0 * ((3.0 * decimal.powi(2)) - (2.0 * decimal.powi(3)));

    let red = spectrum_curve(((spectrum + 2.0) % 6.0) - 2.0);
    let green = spectrum_curve(spectrum - 2.0);
    let blue = spectrum_curve(((spectrum + 4.0) % 6.0) - 2.0);

    let result = (red as u8, blue as u8, green as u8);

    return result;
}

#[inline]
fn depth_to_img(data: &[u16]) -> image::RgbaImage {
    image::RgbaImage::from_fn(640, 480, |x, y| {
        let idx = y * 640 + x;

        let raw_data = data[idx as usize] as f32;

        let depth = convert_raw_kinect_data_to_meters(raw_data);

        if x == 640 / 2 && y == 480 / 2 {
            println!("{}", depth);
            return image::Rgba([255, 255, 255, 255]);
        }

        let (red, blue, green) = convert_depth_to_rgb(depth);

        image::Rgba([red, green, blue, 255])
    })
}

fn push_image_to_window(dstream: &freenect::FreenectDepthStream,
                        mut dimg: image::RgbaImage,
                        dwin: &mut imgwin::ImgWindow) -> image::RgbaImage {
    let _ = imgwin::FixWaitTimer::new(Duration::from_millis(1000 / 25));
    if let Ok((data, _ /* timestamp */)) = dstream.receiver.try_recv() {
        dimg = depth_to_img(&*data);
    }
    dwin.set_img(dimg.clone());
    dwin.redraw();

    return dimg;
}

pub fn main() {
    let freenect_context = freenect::FreenectContext::init_with_video_motor().unwrap();
    let device_count = freenect_context.num_devices().unwrap();

    let device = freenect_context.open_device(0).expect("No device connected - abort");

    device.set_depth_mode(freenect::FreenectResolution::Medium,
                          freenect::FreenectDepthFormat::Bit11).unwrap();

    let mut depth_window = imgwin::ImgWindow::new("Live Depth");

    let depth_stream = device.depth_stream().unwrap();
    let mut depth_image = image::RgbaImage::new(640, 480);
    freenect_context.spawn_process_thread().unwrap();

    loop {
        depth_image = push_image_to_window(&depth_stream, depth_image, &mut depth_window);
    }
}