mod mandelbrot;
mod util;

use clap::Parser;
use num::complex::Complex;
use pbr::ProgressBar;
use std::path::Path;
use std::sync::mpsc::channel;
use threadpool::ThreadPool;
use time::Instant;

use mandelbrot::{get_mandelbrot_color, MandelbrotPoint};

#[derive(Parser, Debug)]
struct Args {
    #[clap(short, long, help="Print verbose output")]
    verbose: bool,

    #[clap(long, help="Generate color image")]
    color: bool,

    #[clap(short, long, default_value="-0.75,0.3", help="Center point of the set to examine")]
    center: String,

    #[clap(short, long, default_value="1000x1000", help="Dimenions of the output image")]
    dimensions: String,

    #[clap(short, long, default_value_t=32, help="Number of threads to use")]
    iterations: u32,

    #[clap(short, long, default_value_t=10, help="Number of threads to use")]
    threads: usize,

    #[clap(short, long, default_value_t=0.5, help="The radius to examine")]
    radius: f32,

    #[clap(default_value="fractal.png", help="Output file name")]
    name: String
}

fn main() {
    let args = Args::parse();

    let (width, height) = util::parse_list(args.dimensions, "x");
    let center: (f32, f32) = util::parse_list(args.center, ",");

    let start = Instant::now();
    let pool = ThreadPool::new(args.threads);
    let (tx, rx) = channel();

    let mut imgbuf = image::ImageBuffer::new(width, height);

    if args.verbose {
        println!(
            "=> Generating output image of size {}x{} at point ({}, {}) with radius {} and iteration depth {}...",
            width, height, center.0, center.1, args.radius, args.iterations
        );
    }

    for x in 0..width {
        for y in 0..height {
            let tx = tx.clone();

            pool.execute(move|| {
                let c = Complex::new(
                    ((x as f32 * args.radius / width as f32) - args.radius / 2.0) + center.0,
                    -((y as f32 * args.radius / height as f32) - args.radius / 2.0) + center.1
                );

                let point = MandelbrotPoint{
                    x: x, y: y,
                    color: get_mandelbrot_color(c, args.iterations, args.color)
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

        if args.verbose && count % 10000 == 0 {
            progress.add(10000);
        }
        count += 1;
    });

    if args.verbose {
        progress.finish();
        println!("=> Saving output image...");
    }

    let _ = image::DynamicImage::ImageRgb8(imgbuf).save(Path::new(&args.name));

    if args.verbose {
        println!("=> Output image saved as '{}'", args.name);
        println!("=> Time taken: {:.2}s", start.elapsed().as_seconds_f64());
    }
}
