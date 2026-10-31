# Upstream boundary

The upstream Horizon source is tracked as the git submodule at
upstream/horizon.

- Upstream: https://github.com/horizon-rust/horizon
- License: MIT for Horizon's original work.
- Pinned revision: recorded by the submodule commit.
- This repository adds Android-only Rust integration around the upstream game.

No Mojang code, assets, jars, decompiled source, or generated resources may be
committed here. The upstream submodule is not edited in place. Android changes
belong in this repository until they are ready to be proposed upstream in
small, separately reviewed patches.
