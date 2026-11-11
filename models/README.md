# Bundled models

The neural networks behind `auto-crop straighten`. Both are compiled into the binary.

- [`UVDoc_grid.onnx`](#uvdoc): UVDoc, the default.
- [`SyntheticDoc_121x177.onnx`](#syntheticdoc): SyntheticDoc's high-resolution model,
  selected with `--model syntheticdoc`.

## UVDoc

| | |
|---|---|
| Model | UVDoc: Neural Grid-based Document Unwarping (Verhoeven, Magne, Sorkine-Hornung; SIGGRAPH Asia 2023) |
| Original weights | [tanguymagne/UVDoc](https://github.com/tanguymagne/UVDoc), `model/best_model.pkl`, MIT licence |
| ONNX export | [fredcallagan/uvdoc-grid-onnx](https://huggingface.co/fredcallagan/uvdoc-grid-onnx), revision `39c8284d324446cfbbba3fd0695c2b3358fefaca`, published under the Apache License 2.0 |
| Input | `image`: `[1, 3, 720, 496]`, RGB in 0..1 |
| Output | `[1, 2, 45, 31]`: grid of source coordinates in -1..1 |

### Changes made here

The export stores its weights in a separate file (`UVDoc_grid.onnx.data`). For this
repository the two files were merged into one self-contained ONNX file, so that it can
be loaded from memory. The network and its weights are unchanged; the merged file
produces output identical to the original pair.

```
sha256  7d094fdf6316928fd1bf2cfd90bb231ff53cc8d59941af1cf5101d0eb4f0ab7f  UVDoc_grid.onnx           (this file)
sha256  044b406398e8accf2b8c896043f22d318221e443bb095c55d184e97f3eb003f7  UVDoc_grid.onnx           (upstream export)
sha256  3a58a04944e59578200d7b0ed02ccb3ade934e9d88efa21454fbb8b53fa40f02  UVDoc_grid.onnx.data      (upstream export)
```

### Licences

The model is not covered by this project's own licence. It is redistributed under the
terms of its sources:

- [`LICENSE-UVDoc-MIT`](LICENSE-UVDoc-MIT): the licence of the original UVDoc code and
  weights, Copyright (c) 2023 Tanguy MAGNE.
- [`LICENSE-onnx-export-Apache-2.0`](LICENSE-onnx-export-Apache-2.0): the licence file
  of the ONNX export.

The test image `tests/data/warped-page.jpg` is a crop of `img/header.jpg` from the
UVDoc repository and falls under the same MIT licence.

### Citation

```bibtex
@inproceedings{UVDoc,
    title={{UVDoc}: Neural Grid-based Document Unwarping},
    author={Floor Verhoeven and Tanguy Magne and Olga Sorkine-Hornung},
    booktitle = {SIGGRAPH ASIA, Technical Papers},
    year = {2023},
    url={https://doi.org/10.1145/3610548.3618174}
}
```

## SyntheticDoc

| | |
|---|---|
| Model | SyntheticDoc: A Large Synthetic Dataset for Document Unwarping and Illumination Correction (Woortmann, Magne, Sorkine-Hornung; ECCV 2026), high-resolution 121×177 model |
| Code | [tanguymagne/SyntheticDoc](https://github.com/tanguymagne/SyntheticDoc), MIT licence |
| Original weights | <https://igl.ethz.ch/projects/SyntheticDoc/highres_121x177.ckpt>, linked from the repository's `training/README.md` |
| Input | `image`: `[1, 3, 1440, 1024]`, RGB in -1..1 |
| Output | `grid_2d`: `[1, 2, 121, 177]`: grid of source coordinates in -1..1 |

### Changes made here

The checkpoint is a PyTorch Lightning file. It was exported to ONNX (opset 17) with the
repository's own model code, keeping only the backward-map output; the shadow-map
decoder is not included. The weights of the exported part are unchanged, and the tool's
result with this file matches that of the repository's `inference.py`.

```
sha256  18fba60d9ee4218ae709f8d6a641753696c647773c1e1a962becb2dd4e63526d  SyntheticDoc_121x177.onnx   (this file)
sha256  a9b69148fb3a8bf309757106f489df17a339a03f298f59d882f5ff51ba65a91d  highres_121x177.ckpt        (original checkpoint)
```

### Licence

The model is not covered by this project's own licence.
[`LICENSE-SyntheticDoc-MIT`](LICENSE-SyntheticDoc-MIT) is the licence of the
SyntheticDoc repository, Copyright (c) 2026 Tanguy MAGNE. The checkpoint is hosted on
the authors' project server and linked from that repository; it carries no separate
licence statement, so it is taken here to fall under the repository's MIT licence.

### Citation

```bibtex
@inproceedings{SyntheticDoc:2026,
    author = {Woortmann, Daniel and Magne, Tanguy and Sorkine-Hornung, Olga},
    title = {{SyntheticDoc}: A Large Synthetic Dataset for Document Unwarping and Illumination Correction},
    booktitle={Computer Vision -- ECCV 2026},
    year = {2026},
}
```
