# Architecture

auto-crop crops every scanned page of a PDF to the sheet of paper visible in the scan.
This document records the main design decisions, why they were made, and what they
cost. For the details, follow the module references into the code.

## Overview

```
main.rs          CLI, simple-scan detection, safe file replacement
simple_scan.rs   recognising simple-scan's postprocessing calling convention
crop.rs          PDF side: find the scan image on each page, map the result to page space, set boxes
image_decode.rs  turn a PDF image XObject into an RGB bitmap
detect.rs        image side: locate the paper inside a bitmap
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
- **Consequence:** no deskew. Rotation would require resampling, which conflicts with
  this decision. If deskew is ever added, it should be opt-in.

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
- **image:** JPEG decoding, resizing, blurring. Only the `jpeg` and `png` features are
  enabled, to keep the build small.
- **kurbo:** affine transforms and rectangles. Its `Affine` uses PDF's `[a b c d e f]`
  coefficient order, so matrices map directly. Note that `a * b` applies `b` first; the
  PDF rule "new CTM = M × CTM" therefore becomes `ctm *= m`.
- **rayon, clap, anyhow:** parallelism, CLI, error context.

Colour handling in `image_decode.rs` is deliberately approximate: no colour management,
`/Decode` arrays or soft masks. Detection only has to tell paper from lid, and it does
not depend on exact colours.
