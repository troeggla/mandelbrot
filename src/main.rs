extern crate argparse;
extern crate image;
extern crate num;
extern crate pbr;
extern crate threadpool;
extern crate time;

use argparse::{ArgumentParser, Store, StoreTrue};
use num::complex::Complex;
use pbr::ProgressBar;
use std::fs::File;
use std::path::Path;
use std::sync::mpsc::channel;
use threadpool::ThreadPool;
use time::PreciseTime;

struct MandelbrotPoint {
    x: u32,
    y: u32,
    color: image::Rgb<u8>
}

fn mandelbrot(z: Complex<f32>, c: Complex<f32>) -> Complex<f32> {
    num::pow(z, 2) + c
}

fn in_mandelbrot_set(c: Complex<f32>, iterations: i32) -> (bool, i32) {
    let mut z = c;

    for i in 0..iterations {
        if num::pow(z.re, 2) + num::pow(z.im, 2) > 4.0 {
            return (false, i);
        }

        z = mandelbrot(z, c);
    }

    (true, iterations)
}

fn get_greyscale_pixel(ratio: f32) -> image::Rgb<u8> {
    let color = (ratio * 255.0) as u8;

    image::Rgb([
        color,
        color,
        color
    ])
}

fn get_color_pixel(ratio: f32) -> image::Rgb<u8> {
    let color_value = (ratio * 0xFFFFFF as f32) as u32;

    let r = ((color_value & 0xFF0000) >> 16) as u8;
    let g = ((color_value & 0x00FF00) >> 8) as u8;
    let b = (color_value & 0x0000FF) as u8;

    image::Rgb([r, g, b])
}

fn get_mandelbrot_color(c: Complex<f32>, iterations: i32) -> image::Rgb<u8> {
    let (in_set, iterations_taken) = in_mandelbrot_set(c, iterations);

    if in_set {
        image::Rgb([0, 0, 0])
    } else {
        get_greyscale_pixel(iterations_taken as f32 / iterations as f32)
    }
}

fn main() {
    let mut verbose = false;
    let mut width = 1000;
    let mut height = 1000;
    let mut iterations = 250;

    {
        let mut ap = ArgumentParser::new();

        ap.set_description("Renders images of portions of the Mandelbrot set.");
        ap.refer(&mut verbose)
          .add_option(&["-v", "--verbose"], StoreTrue, "Enable verbose output");
        ap.refer(&mut width)
          .add_option(&["-w", "--width"], Store, "Output image width (default 1000)");
        ap.refer(&mut height)
          .add_option(&["-h", "--height"], Store, "Output image height (default 1000)");
        ap.refer(&mut iterations)
          .add_option(&["-i", "--iterations"], Store, "Number of iterations (default 250)");

        ap.parse_args_or_exit();
    }

    let start = PreciseTime::now();
    let num_threads = 10;

    let center: (f32, f32) = (-0.75, 0.3);
    let r: f32 = 0.5;

    let pool = ThreadPool::new(num_threads);
    let (tx, rx) = channel();
    let mut imgbuf = image::ImageBuffer::new(width, height);

    if verbose {
        println!("Generating Mandelbrot set of size {}x{} with iteration depth {}...", width, height, iterations);
    }

    for x in 0..width {
        for y in 0..height {
            let tx = tx.clone();

            pool.execute(move|| {
                let c = Complex::new(
                    ((x as f32 * r / width as f32) - r / 2.0) + center.0,
                    -((y as f32 * r / height as f32) - r / 2.0) + center.1
                );

                let point = MandelbrotPoint{
                    x: x, y: y,
                    color: get_mandelbrot_color(c, iterations)
                };

                tx.send(point)
                  .expect("Could not send");
            });
        }
    }

    let mut progress = ProgressBar::new((width * height) as u64);
    let mut count = 0;

    rx.iter().take((width * height) as usize).for_each(|point| {
        if point.color != image::Rgb([0, 0, 0]) {
            imgbuf.put_pixel(point.x, point.y, point.color);
        }

        if verbose && count % 10000 == 0 {
            progress.add(10000);
        }
        count += 1;
    });

    if verbose {
        progress.finish();
        println!("Generating output...");
    }

    let ref mut fname = File::create(&Path::new("fractal.png")).unwrap();
    let _ = image::ImageRgb8(imgbuf).save(fname, image::PNG);

    if verbose {
        let end = PreciseTime::now();
        println!("Time taken: {}s", start.to(end).num_seconds());
    }
}
