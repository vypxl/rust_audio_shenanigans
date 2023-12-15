# Rust audio shenanigans

This repo contains my adventures in using Rust for audio synthesis and processing.

## Try it

If you want to try this, go ahead and get some midi files.
You can start the application with `cargo run`, or optionally
`cargo run -- song.mid` to specify a song to play.
You will see a simple gui with a file picker, start and stop buttons.

The midi file you want to play will be performed using a simple polyphonic
square wave based instrument.

## What is this about?

I wanted to write a synthesizer in Rust. One that makes easy to create
any sound you want, by additive or substractive synthesis, or even frequency modulation.

Some examples:

```rust
constant(440) >> square(); // A 440hz square wave

// A 800hz square wave that wobbles slightly at 2hz
(constant(800) + (constant(2) >> sine())) >> (square() * 0.5)
```

A simple instrument made up of 8 voices. This one is not complete, it needs an input.

```rust
let wave = triangle();
let wave2 = ((pass() * 2) >> square()) * 0.5;
let wave3 = ((pass() * 3) >> square()) * 0.25;
let wave4 = ((pass() * 4) >> square()) * 0.125;
let wave5 = ((pass() * 5) >> square()) * 0.0625;
let wave6 = ((pass() * 6) >> square()) * 0.03125;
let wave7 = ((pass() * 7) >> square()) * 0.015625;
let wave8 = ((pass() * 8) >> square()) * 0.0078125;

((wave + wave2 + wave3 + wave4 + wave5 + wave6 + wave7 + wave8) * 0.2) >> lowpass(5000.0, 1.0)
```

Waves can be combined using operators like `*` for multiplication, `+` for
addition, `>>` for chaining (lhs determines the frequency of lhs).

The `PolyInstrument` struct takes a wave that needs an input and uses that
to play it with any source pitch, enabling usage like a midi synthesizer.

To play around different sounds, you can edit `src/player.rs`. At the top of the
file resides the definition for the sound that the program uses to play a file.
