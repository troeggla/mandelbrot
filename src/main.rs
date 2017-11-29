extern crate image;
extern crate num;
extern crate threadpool;
extern crate time;

use num::complex::Complex;
use std::fs::File;
use std::path::Path;
use std::sync::mpsc::channel;
use threadpool::ThreadPool;
use time::PreciseTime;

struct MandelbrotPoint {
    x: u32,
    y: u32,
    pixel: image::Rgb<u8>,
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

fn main() {
    let start = PreciseTime::now();
    let iterations = 250;
    let num_threads = 10;

    let (width, height) = (1000, 1000);
    let center: (f32, f32) = (-0.75, 0.3);
    let r: f32 = 0.5;

    let pool = ThreadPool::new(num_threads);
    let (tx, rx) = channel();
    let mut imgbuf = image::ImageBuffer::new(width, height);

    for x in 0..width {
        for y in 0..height {
            let tx = tx.clone();

            pool.execute(move|| {
                let c = Complex::new(
                    ((x as f32 * r / width as f32) - r / 2.0) + center.0,
                    -((y as f32 * r / height as f32) - r / 2.0) + center.1
                );

                let (in_set, iterations_taken) = in_mandelbrot_set(c, iterations);
                let pixel: image::Rgb<u8>;

                if in_set {
                    pixel = image::Rgb([0, 0, 0]);
                } else {
                    let color = ((255 / iterations * 2) * iterations_taken) as u8;

                    pixel = image::Rgb([
                        color,
                        color,
                        color
                    ]);
                }

                tx.send(MandelbrotPoint{ x: x, y: y, pixel: pixel })
                  .expect("Could not send");
            });
        }
    }

    let mut count = 0;
    rx.iter().take((width * height) as usize).for_each(|point| {
        if count % 10000 == 0 {
            println!("Processing point {}/{}: x:{} y:{} rgb:{:?}", count, width * height, point.x, point.y, point.pixel);
        }

        imgbuf.put_pixel(point.x, point.y, point.pixel);
        count += 1;
    });

    println!("Generating output...");

    let ref mut fname = File::create(&Path::new("fractal.png")).unwrap();
    let _ = image::ImageRgb8(imgbuf).save(fname, image::PNG);

    let end = PreciseTime::now();
    println!("Time taken: {}s", start.to(end).num_seconds());
}
