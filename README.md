# Wittgenstein's Nachlass Facsimiles

This tool downloads all the facsimiles of Wittgenstein's Nachlass that are kept
in the Wren Library and released under a
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

Please note that while the software is released under the MIT license, the
facsimile files are released under a Creative Commons license (see above).

## How to run locally

Install [Rust](https://www.rust-lang.org/tools/install), then:

```
cargo run --release
```

This downloads all CC-licensed Wren Library facsimiles as PNG images into
`facsimiles/2000px/png/`, one subdirectory per document. WebP copies are
automatically created in `facsimiles/2000px/webp/`.

You can also install the binary globally:

```
cargo install --path .
wittgensteinsourcerer
```

### Options

```
Download Wittgenstein Nachlass facsimiles from wittgensteinsource.org

Usage: wittgensteinsourcerer [OPTIONS]

Options:
      --max-width <MAX_WIDTH>      Maximum image width in pixels [default: 2000]
      --destination <DESTINATION>  Base output directory [default: facsimiles/{max_width}px]
      --all                        Download all facsimiles, not just CC-licensed ones from the Wren Library
      --skip <SKIP>                Skip these documents (comma-separated names, e.g. Ms-107,Ms-108)
      --only <ONLY>                Only download these documents (comma-separated names, e.g. Ms-107,Ms-108)
      --parallel <PARALLEL>        Number of images to download in parallel [default: 1]
      --pdf [<PDF>]                Generate a PDF for each document. Use --pdf for JPEG q90 (default), --pdf=75 for custom quality, or --pdf=uncompressed for raw RGB
      --webp-slow                  Disable parallel WebP conversion
  -h, --help                       Print help
```

### Examples

Download everything at default resolution:

```
cargo run --release
```

Download only Ms-101 and Ms-102 with 5 parallel downloads:

```
cargo run --release -- --only Ms-101,Ms-102 --parallel 5
```

Download at higher resolution and generate JPEG-compressed PDFs (q90):

```
cargo run --release -- --max-width 5000 --pdf
```

Generate PDFs with custom JPEG quality:

```
cargo run --release -- --pdf=75 --only Ms-101
```

Generate uncompressed PDFs (large files):

```
cargo run --release -- --pdf=uncompressed --only Ms-101
```

### Copyright

By default, this tool only downloads facsimiles from the Wren Library that are
released under a CC BY-NC 4.0 license. If you use the `--all` flag, you will
also download facsimiles that may be under different or more restrictive
copyright terms. It is your responsibility to verify that you have the legal
right to download and use those files in your jurisdiction. This tool does not
grant any rights to the downloaded material.

### Be nice to the server

The `--parallel` flag defaults to 1 for a reason: wittgensteinsource.org is a
small academic server, not a CDN. Please stick to the default of sequential
downloads unless you have a good reason not to.
