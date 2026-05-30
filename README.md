# KeyDraw

KeyDraw is a Rust graphics engine. It's very simple, and has one job: Enable the drawing of many different types of objects so that the programmer doesn't have to worry about that stuff. _Hopefully_, it does that job well.

It wraps [wgpu](https://github.com/gfx-rs/wgpu) and [winit](https://github.com/rust-windowing/winit). To use this, you don't have to know about how `winit` works, but you will need to know about `wgpu`, specifically designing pipelines and bind groups. Additionally, you should know how to write `wgsl` shaders, and about rendering concepts like buffers, indexing, instancing, etc.

This project is still early in development, although it's feature-complete within the intended scope, meaning it is technically usable, and I intend to use it for future rendering projects.

## Examples?

Examples are located in the `examples` directory, they can be ran with `cargo run --example [name]`. Note that I haven't tested these examples on non-linux, non-wayland environments, so they might not work.
