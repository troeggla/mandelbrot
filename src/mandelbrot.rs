use num::complex::Complex;

pub struct MandelbrotPoint {
    pub x: u32,
    pub y: u32,
    pub color: image::Rgb<u8>
}

fn mandelbrot(z: Complex<f32>, c: Complex<f32>) -> Complex<f32> {
    num::pow(z, 2) + c
}

pub fn in_mandelbrot_set(c: Complex<f32>, iterations: u32) -> (bool, u32) {
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

pub fn get_mandelbrot_color(c: Complex<f32>, iterations: u32, color: bool) -> image::Rgb<u8> {
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
