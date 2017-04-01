extern crate image;
extern crate num;

use num::complex::Complex;
use std::fs::File;
use std::path::Path;

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
    let iterations = 250;

    let (width, height) = (5000, 5000);
    let center: (f32, f32) = (-0.75, 0.3);
    let r: f32 = 0.5;

    let mut imgbuf = image::ImageBuffer::new(width, height);

    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let c = Complex::new(
            ((x as f32 * r / width as f32) - r / 2.0) + center.0,
            -((y as f32 * r / height as f32) - r / 2.0) + center.1
        );

        if y % 100 == 0 && x == 0 {
            println!("Iterating row {} {}", x, y);
        }

        let (in_set, iterations_taken) = in_mandelbrot_set(c, iterations);

        if in_set {
            *pixel = image::Rgb([0, 0, 0]);
        } else {
            let color = ((255 / iterations * 2) * iterations_taken) as u8;

            *pixel = image::Rgb([
                color,
                color,
                color
            ]);
        }
    }

    println!("Generating output...");

    let ref mut fname = File::create(&Path::new("fractal.png")).unwrap();
    let _ = image::ImageRgb8(imgbuf).save(fname, image::PNG);
}
