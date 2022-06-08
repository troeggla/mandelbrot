extern crate argparse;
extern crate image;
extern crate num;
extern crate pbr;
extern crate threadpool;
extern crate time;

mod mandelbrot;
mod util;

use argparse::{ArgumentParser, Store, StoreTrue};
use mandelbrot::{get_mandelbrot_color, MandelbrotPoint};
use num::complex::Complex;
use pbr::ProgressBar;
use std::path::Path;
use std::sync::mpsc::channel;
use threadpool::ThreadPool;
use time::Instant;

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
        let help_texts = vec![
            format!("Enable verbose output"),
            format!("Generate output image in colour"),
            format!("Output image dimensions, separated by 'x' (default {})", dimensions),
            format!("Centre point of the set separated by comma (default {})", center),
            format!("Radius of the set to be examined (default {})", r),
            format!("Number of iterations (default {})", iterations),
            format!("Number of threads to spawn (default {})", num_threads),
            format!("Output file name (default '{}')", fname)
        ];

        let mut ap = ArgumentParser::new();

        ap.set_description("Renders images of portions of the Mandelbrot set.");

        ap.refer(&mut verbose).add_option(&["-v", "--verbose"], StoreTrue, &help_texts[0]);
        ap.refer(&mut color).add_option(&["--color"], StoreTrue, &help_texts[1]);
        ap.refer(&mut dimensions).add_option(&["-s", "--size"], Store, &help_texts[2]);
        ap.refer(&mut center).add_option(&["-c", "--center"], Store, &help_texts[3]);
        ap.refer(&mut r).add_option(&["-r", "--radius"], Store, &help_texts[4]);
        ap.refer(&mut iterations).add_option(&["-i", "--iterations"], Store, &help_texts[5]);
        ap.refer(&mut num_threads).add_option(&["-t", "--threads"], Store, &help_texts[6]);
        ap.refer(&mut fname).add_option(&["-f", "--fname"], Store, &help_texts[7]);

        ap.parse_args_or_exit();
    }

    let (width, height) = util::parse_list(dimensions, "x");
    let center: (f32, f32) = util::parse_list(center, ",");

    let start = Instant::now();
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

    let _ = image::DynamicImage::ImageRgb8(imgbuf).save(Path::new(&fname));

    if verbose {
        println!("=> Output image saved as '{}'", fname);
        println!("=> Time taken: {:.2}s", start.elapsed().as_seconds_f64());
    }
}
