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

This downloads all CC-licensed Wren Library facsimiles as PNG images into
`facsimiles-2000px/`, one subdirectory per document. WebP copies are
automatically created in `facsimiles-2000px-webp/`.

### Options

| Flag | Description | Default |
|------|-------------|---------|
| `--max-width <PX>` | Maximum image width in pixels | `2000` |
| `--destination <DIR>` | Output directory | `facsimiles-{max_width}px` |
| `--all` | Download all facsimiles, not just CC-licensed Wren Library ones | off |
| `--only <LIST>` | Only download these documents (comma-separated, e.g. `Ms-107,Ms-108`) | all |
| `--skip <LIST>` | Skip these documents (comma-separated, e.g. `Ms-107,Ms-108`) | none |
| `--parallel <N>` | Number of images to download in parallel | `1` |
| `--pdf[=QUALITY]` | Generate a PDF for each document. `--pdf` uses JPEG q90, `--pdf=75` uses q75, `--pdf=uncompressed` embeds raw RGB | off |

`--only` and `--skip` cannot be used together.

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