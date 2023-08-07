# Performance

An experiment on how dynamic dispatch worsens performance.

I have two benchmarks. One plays a midi file with a PolyInstrument, the other just produces a filtered sine wave.

## Static dispatch

On my desktop on commit 66f6a19a0f089d6ba67e332e42760c95594901ce:
Bench mountain_king: 227ns / sample
About 100x speed compared to real time

Bench sine_lowpass: 14.577ns / sample

## Dynamic dispatch

After porting to use dynamic dispatch everywhere:

```plain
mountain_king/mountain_king
                        time:   [50.951 ns 53.806 ns 56.365 ns]
                        change: [+32.152% +38.235% +44.939%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 8 outliers among 100 measurements (8.00%)
  8 (8.00%) high mild

sine_lowpass/sine_lowpass
                        time:   [20.001 ns 20.530 ns 21.178 ns]
                        change: [+46.183% +53.614% +62.291%] (p = 0.00 < 0.05)
                        Performance has regressed.
```
