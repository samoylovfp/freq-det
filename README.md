# Frequency detector

```rust
let freq: f32 = freq_det::FreqDetector::new(44100, 4096).detect(&samples);
```
It is that easy!

Consult with [from_mic.rs](examples/from_mic.rs) to see how microphone sound
can be analyzed.

## Contributions
PRs are welcome!
