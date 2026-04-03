# Wittgenstein's Nachlass Facsimiles

This repository allows you to download all the facsimiles of Wittgenstein's
Nachlass that are kept in the Wren Library and released under a
[Creative Commons Attribution-NonCommercial 4.0 International License](https://creativecommons.org/licenses/by-nc/4.0/).

> CC BY-NC 4.0. Original at the Wren Library, Trinity College, Cambridge, where
> in 2014-15, on the request of the Wittgenstein Archives at the University of
> Bergen (WAB) and with the generous financial support of the Stanhill
> Foundation, London, this scan was produced. The image was post-processed at
> WAB and is reproduced here by permission of The Master and Fellows of Trinity
> College, Cambridge, and the University of Bergen, Bergen. The sale, further
> reproduction or use of this image for commercial purposes without prior
> permission from the copyright holder is prohibited. © 2015 The Master and
> Fellows of Trinity College, Cambridge; The University of Bergen, Bergen

For more details, see http://www.wittgensteinsource.org

Please note that while the software used to fetch the facsimiles is released
under the MIT license, the facsimile files are released under a Creative
Commons license (see [facsimiles/LICENSE](facsimiles/LICENSE)).

For demonstration purposes, the result of downloading Ms-101 with a maximum
width of 2000 px is included in the [facsimiles](facsimiles/) directory.

## How to run locally

Install [Rust](https://www.rust-lang.org/tools/install), then:

```
cargo run --release
```

By default, the tool only downloads facsimiles kept in the Wren Library, as
only these are released under a CC BY-NC license. To download all facsimiles
from the collection (regardless of license), pass `--all`:

```
cargo run --release -- --all
```

You can also customize the image resolution and destination directory:

```
cargo run --release -- --max-width 5000 --destination path/to/my/directory
```