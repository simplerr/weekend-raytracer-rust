![Image](image.png)

An implementation of Peter Shirley's [Ray Tracing in One Weekend](https://raytracing.github.io/books/RayTracingInOneWeekend.html) in Rust.

## Building and running

`cargo run --release > image.ppm ; ffmpeg -y -i image.ppm image.png`
