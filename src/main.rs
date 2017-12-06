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

fn parse_list<T: std::str::FromStr>(dimensions: String, delimiter: &str) -> (T, T) {
    let mut result: Vec<T> = dimensions.split(delimiter).take(2).map(|s| {
        s.parse::<T>().ok().unwrap()
    }).collect();

    (result.remove(0), result.remove(0))
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

fn get_mandelbrot_color(c: Complex<f32>, iterations: i32, color: bool) -> image::Rgb<u8> {
    let (in_set, iterations_taken) = in_mandelbrot_set(c, iterations);

    if in_set {
        image::Rgb([0, 0, 0])
    } else {
        if color {
            get_color_pixel(iterations_taken as f32 / iterations as f32)
        } else {
            get_greyscale_pixel(iterations_taken as f32 / iterations as f32)
        }
    }
}

fn main() {
    let mut verbose = false;
    let mut color = false;
    let mut dimensions = "1000x1000".to_string();
    let mut iterations = 250;
    let mut num_threads = 10;
    let mut center = "-0.75,0.3".to_string();
    let mut r: f32 = 0.5;
    let mut fname = "fractal.png".to_string();

    {
        let mut ap = ArgumentParser::new();

        ap.set_description("Renders images of portions of the Mandelbrot set.");
        ap.refer(&mut verbose)
          .add_option(&["-v", "--verbose"], StoreTrue, "Enable verbose output");
        ap.refer(&mut color)
          .add_option(&["-c", "--color"], StoreTrue, "Generate output image in colour");
        ap.refer(&mut dimensions)
          .add_option(&["-s", "--size"], Store, "Output image dimensions, separated by space (default 1000x1000)");
        ap.refer(&mut center)
          .add_option(&["-c", "--center"], Store, "Centre point of the set (default -0.75,0.3)");
        ap.refer(&mut r)
          .add_option(&["-r", "--radius"], Store, "Radius of the set to be examined (default 0.5)");
        ap.refer(&mut iterations)
          .add_option(&["-i", "--iterations"], Store, "Number of iterations (default 250)");
        ap.refer(&mut num_threads)
          .add_option(&["-t", "--threads"], Store, "Number of threads to spawn (default 10)");
        ap.refer(&mut fname)
          .add_option(&["-f", "--fname"], Store, "Output file name (default 'fractal.png')");

        ap.parse_args_or_exit();
    }

    let (width, height) = parse_list(dimensions, "x");
    let center: (f32, f32) = parse_list(center, ",");

    let start = PreciseTime::now();
    let pool = ThreadPool::new(num_threads);
    let (tx, rx) = channel();

    let mut imgbuf = image::ImageBuffer::new(width, height);

    if verbose {
        println!(
            "=> Generating output image of size {}x{} at point ({}, {}) with radius {} and iteration depth {}...",
            width, height, center.0, center.1, r, iterations
        );
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
                    color: get_mandelbrot_color(c, iterations, color)
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
        println!("=> Saving output image...");
    }

    let ref mut outfile = File::create(&Path::new(&fname)).unwrap();
    let _ = image::ImageRgb8(imgbuf).save(outfile, image::PNG);

    if verbose {
        let end = PreciseTime::now();

        println!("=> Output image saved as '{}'", fname);
        println!("=> Time taken: {}s", start.to(end).num_seconds());
    }
}
