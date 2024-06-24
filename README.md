# Frequency detector

```rust
use freq_det::FreqDetector;

let detector = FreqDetector::new(44100, 4096).unwrap();
let freq: f32 = detector.detect(&samples).unwrap();
```

It is that easy!

Consult with [from_mic.rs](examples/from_mic.rs) to see how microphone sound
can be analyzed.

## Contributions
PRs are welcome!
