# Seeded HashWX / HashX reference workload

This exhibit preserves the production release-gate references rather than conflating them with the recovered fixed-program benchmark. Revisions and corpus sizes are pinned in `references.toml`.

The canonical C implementations and `hashwx-rs` execute identical seed/nonces corpora in adjacent A/B chunks. HashWX uses 3,000 seeds × 463 nonces; HashX uses 500 seeds × 65,536 nonces. Five chunks and three alternating-order sweeps reduce drift. This is the right workload for native/reference ratios and compile/generation variance; `hashwx-fixed-v1` is the right workload for comparing one emitted instruction stream across language runtimes.

An experimental seeded GPU backend is documented in [zipp-gpu](zipp-gpu/README.md). It uses exact integer WGSL and Zipp’s native WebGPU shim; its pilot measurements are separate from the canonical release gate.
