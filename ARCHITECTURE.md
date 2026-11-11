# Architecture

auto-crop crops every scanned page of a PDF to the sheet of paper visible in the scan.
This document records the main design decisions, why they were made, and what they
cost. For the details, follow the module references into the code.

## Overview

```
main.rs          CLI, simple-scan detection, safe file replacement
simple_scan.rs   recognising simple-scan's postprocessing calling convention
pdf_page.rs      PDF side: find the scan image on a page and its placement, set page boxes
image_decode.rs  turn a PDF image XObject into an RGB bitmap
crop.rs          crop mode: map the detected paper to page space
detect.rs        image side: locate the paper inside a bitmap
straighten.rs    straighten mode: image files and PDF pages
unwarp.rs        image side: run the UVDoc model, resample along its grid
whiten.rs        image side: even out the lighting (optional step of straighten)
```

Data flows in one direction:

```
PDF page ──crop.rs──▶ image XObject + transform ──image_decode.rs──▶ RgbImage
        ◀──crop.rs── page-space box ◀── fractional box ◀──detect.rs──┘
```

`detect.rs` knows nothing about PDFs and `crop.rs` knows nothing about pixels. The two
meet at `FracRect`, a box expressed as fractions of the image size. This keeps the
detection algorithm testable with plain synthetic bitmaps, and independent of how an
image is placed on the page.

## Decisions

### 1. Crop by setting page boxes, never by re-encoding

The output keeps the original image streams byte for byte. Only `/MediaBox` and
`/CropBox` of each cropped page are changed (`crop.rs`, `apply_crop`).

- **Why:** cropping is lossless and fast, the file size stays essentially the same, and
  any OCR text layer stays aligned with the image.
- **Cost:** the hidden image area is still stored in the file, and a viewer that ignores
  the boxes would show the full scan. Both `/MediaBox` and `/CropBox` are set so that
  even tools honouring only one of them crop correctly. `/BleedBox`, `/TrimBox` and
  `/ArtBox` are removed rather than recomputed, because they must lie within the media
  box.
- **Consequence:** no deskew in this mode. Rotation requires resampling, which
  conflicts with this decision. Straightening is therefore a separate, explicitly
  invoked mode (decision 10).

### 2. Native Rust PDF handling (lopdf) instead of a rendering engine

Pages are not rasterised. The tool parses content streams itself, finds the image
XObject that makes up the scan, and decodes only that image.

- **Why:** no runtime dependency on pdfium or MuPDF (MuPDF would bring a heavy C build
  and AGPL licensing), and decoding one image is much cheaper than rendering a page.
- **Cost:** only image encodings the tool can decode are supported: JPEG (`DCTDecode`)
  and raw samples behind generic filters (Flate, LZW, …). JPEG 2000, CCITT fax and
  JBIG2 are not supported, and such pages are skipped with a message. Scans split into
  several image strips are not recognised either.

### 3. "Picture page" means one image covers most of the page

A page is processed if a single image covers at least `MIN_IMAGE_COVERAGE` (50 %) of its
visible area. The transform is tracked through `q`/`Q`/`cm` and into form XObjects, so
wrapped images are found too (`crop.rs`, `collect_images`).

- **Why:** this is robust against OCR text layers, which scanning tools add as invisible
  text on top of the image. "Contains no text" would wrongly skip those pages.
- **Cost:** the heuristic is coverage based. A born-digital page with a full-page
  background image would be treated as a scan; it is then usually left alone because no
  distinct paper is found.

### 4. Paper detection: border colours, flood fill, brightness rule

`detect.rs` works on a copy shrunk to at most 1000 px (`WORK_SIZE`), lightly blurred:

1. Collect the significant colours along the image border (up to 4 clusters, each at
   least 3 % of the border).
2. Treat the darkest *k* of them together as background, starting with all of them.
   Flood fill from the border through matching pixels.
3. The largest unfilled, connected region is the paper candidate. It is accepted if it
   is large enough, brighter than every background colour used, and does not fill the
   whole image. Otherwise *k* is reduced and step 2 is retried.

Reasons behind the individual choices:

- **Flood fill from the border rather than global thresholding:** the background is
  defined as what surrounds the paper. Dark text or photos on the paper can never be
  mistaken for background, because they are not connected to the border.
- **Several background colours at once:** real lids are not uniform. Shading, scanner
  padding and lid strips produce several tones (the bundled `example.pdf` has a flat
  grey bottom and lighter shaded sides).
- **Darkest first, brightness rule:** the paper often touches some edges, so the most
  common border colour can be the paper itself. Assuming *paper is brighter than the
  lid* settles which border colours are background. It also rejects degenerate
  results, such as a dark photo on a borderless page being taken for the paper.
- **Largest connected component after a small erosion:** dust specks and thin lines
  along the scan edge are ignored.

Known limitation: white paper on a lid that is even brighter is not detected. Such
pages are left uncropped rather than cropped wrongly.

### 5. Coarse-to-fine edges

The shrunk image only gives the box to about 1 pt. Each edge is then refined on the
full-resolution image within a small window: the outermost row or column where at least
`EDGE_PAPER_SHARE` (50 %) of the pixels are brighter than every background tone becomes
the edge (`detect.rs`, `refine_edges`).

- **Why:** this removes lid slivers and the dark shadow line the paper's edge casts,
  which the coarse pass cannot resolve.
- **Trade-off:** on a slightly tilted sheet the 50 % rule cuts through the middle of
  the tilt, leaving a hair of lid on one part of an edge and trimming a hair of paper
  on another. For noticeably tilted sheets no line reaches 50 % within the window, and
  the coarse box, which contains the whole sheet, is kept. Corners are never cut off;
  instead grey wedges remain.

### 6. Fail per page, not per document

Each page gets a `PageOutcome` (cropped, not a picture, no paper found, nothing to crop,
failed). An error on one page, such as an unsupported encoding, is reported and that
page is left untouched; the rest of the document is still processed. Crops trimming
less than `MIN_TRIM` (1 pt) on every side are not applied.

Pages are analysed in parallel with rayon. The document is only read during analysis,
and all crops are applied sequentially afterwards, so no locking is needed.

### 7. Never leave a half-written file

Output is written to a hidden temporary file in the target directory and renamed into
place (`main.rs`, `save_replacing`). This makes `input == output` safe, which the
simple-scan integration relies on. When nothing was cropped and input equals output,
the file is not rewritten at all.

### 8. simple-scan integration by argument shape

GNOME Document Scanner calls its postprocessing script as
`<script> <mime type> <true|false> <file> <extra arguments>`. There is no way to insert
a subcommand before these arguments, so `simple_scan.rs` recognises this shape (a MIME
type followed by `true` or `false`) before normal CLI parsing.

- **Why:** the binary can be configured directly as the script, with no wrapper.
- **Behaviour:** the saved file is cropped in place. With *keep original*, the original
  is first copied to `<name>-original.<ext>`, so the name the user chose always holds
  the result. Non-PDF formats exit successfully without changes. simple-scan only logs
  the script's output and warns on a non-zero exit status, so errors are reported
  through the exit code.

### 9. Libraries for the generic parts

- **lopdf:** PDF object model, content stream parsing, generic stream filters.
- **image:** decoding, encoding, resizing, blurring. Only the JPEG, PNG, TIFF and WebP
  features are enabled, to keep the build small.
- **kurbo:** affine transforms and rectangles. Its `Affine` uses PDF's `[a b c d e f]`
  coefficient order, so matrices map directly. Note that `a * b` applies `b` first; the
  PDF rule "new CTM = M × CTM" therefore becomes `ctm *= m`.
- **tract:** pure Rust ONNX inference for the straighten mode (decision 10).
- **rayon, clap, anyhow:** parallelism, CLI, error context.

Colour handling in `image_decode.rs` is deliberately approximate: no colour management,
`/Decode` arrays or soft masks. Detection only has to tell paper from lid, and it does
not depend on exact colours.

### 10. Straightening is a separate mode built on UVDoc and tract

`auto-crop straighten` flattens crooked, photographed or curled documents. It is never
run implicitly (not by the default mode and not by the simple-scan integration),
because it gives up decision 1: the image is resampled and re-encoded.

- **Model:** [UVDoc](https://github.com/tanguymagne/UVDoc) looks at a 496×720 copy of
  the image and predicts a 45×31 grid of points: where a regular grid on the flat
  document lies in the image. A grid, unlike a model that outputs the rectified image,
  can be applied to the full resolution image.
- **Inference with tract:** tract is a pure Rust ONNX runtime, so the binary stays
  statically linked and free of C dependencies (ONNX Runtime would add a native
  library). Cost: tract adds about 25 MB to the stripped binary, and
  inference is CPU only (about 0.2 s per page).
- **The model is built into the binary** with `include_bytes!` (`models/`, about
  32 MB), so the single static binary works without a download or a data directory. The upstream export keeps its weights in a second file,
  which tract can only resolve on disk, so the two files were merged once into a
  self-contained ONNX file with identical output. Cost: the 32 MB file is committed to
  the repository, and the binary grows to about 60 MB. The weights keep their own
  licences (`models/README.md`).
- **Resampling (`unwarp.rs`):** the output size is the document's size measured along
  the grid lines in image pixels, so a scan keeps its resolution and aspect ratio. The
  grid is interpolated with Catmull-Rom splines (smooth, and exact for plain rotations)
  and the image is sampled bicubically.
- **Margins for paper at the image edge (`straighten.rs`):** UVDoc is trained on photos
  with background all around the document. Where paper runs to the edge of the image,
  as on a flatbed scanner, it looks for the edge further inside and cuts content off.
  The crop mode's detector (decision 4) tells which edges the paper touches; on those
  sides the model's input gets a margin in the lid's colour. The detector is made for
  scans and can misreport photos, where a wrong margin makes the model include the
  background; a side therefore only gets a margin if the pixels along that image edge
  also have the paper's colour. Only the small model input
  is padded, and predicted points are clamped back onto the image.
- **PDF pages:** the image stream is replaced by a JPEG of the straightened image. The
  page's content stream is not edited; it is wrapped in a transformation that maps the
  image's old rectangle onto the new page, which is then resized to keep the scan's
  resolution. Images placed at an angle other than a multiple of 90° are skipped. A
  text layer from OCR is carried along but no longer matches the straightened image.
- **Known weakness:** edges are less precise than in crop mode. The model works at low
  resolution, and on scans where the paper touches the image edge the opposite edges
  can keep a wavy sliver of background.

### 11. Whitening is a classical flat-field correction, and opt-in

`straighten --whiten` removes shading and fold shadows after unwarping (`whiten.rs`).
The bare paper is estimated on a copy of at most 1000 px as the brightest colour within
a 41 px square around each point, blurred; the full resolution image is then divided by
that estimate, per channel.

- **Why not a model:** document enhancement networks are much heavier, slow on CPU at
  scan resolution, and can alter fine detail. Dividing by a smooth estimate cannot
  invent content, needs no weights, and takes well under a second.
- **Why opt-in:** anything dark and larger than the square is taken for shaded paper
  and washed out. That is fine for drawings and text, but not for pages with photos.
- **Limits:** within the square's radius of a shadow's edge the brighter side
  dominates the estimate, so narrow shadows and the crease of a fold are only reduced.
  Geometry is untouched.

### 12. SyntheticDoc as a second built-in model

`straighten --model syntheticdoc` uses the high-resolution model of
[SyntheticDoc](https://github.com/tanguymagne/SyntheticDoc) (ECCV 2026, by UVDoc's
authors) instead of UVDoc.

- **What it is:** UVDoc's architecture with a 1024×1440 input, trained on a million
  simulated samples that include folds and crumples. Its 121×177 output is a bilinear
  upsampling of a 64×90 prediction, so the real resolution is about twice UVDoc's per
  axis. The model also predicts a shadow map; only the grid output was exported, since
  `--whiten` did better on the test photo.
- **Why optional and not the default:** on the test images it crops tighter and leaves
  no background, but trims a few percent off the page's edge, while UVDoc errs on the
  side of keeping background. Losing content is the worse failure, so UVDoc stays the
  default. It does not remove crease kinks either, which a denser grid alone cannot do.
- **Integration:** both models share one contract (RGB image in, grid of normalized
  source coordinates out), so everything after the prediction is unchanged. They differ
  in input size, grid size, and the input range (`0..1` for UVDoc, `-1..1` for
  SyntheticDoc), which `unwarp::Model` encapsulates. Loading models from a file is no
  longer offered, as a file does not say which input range it expects.
- **Cost:** another 32 MB in the repository and the binary.
