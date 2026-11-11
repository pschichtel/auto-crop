# Licences, pretrained weights, ONNX availability and tract compatibility of UVDoc alternatives

Facts as of 2026-11-11. "Verified" means I opened the LICENSE file, README, source file or API record in this session. Architecture/op statements marked "from prior knowledge" were not re-checked against model source in this session. Not legal advice.

## 1. Licence of code and weights per model; GPL-3.0 redistributability

### Takeaway
Only the Stony Brook models (DewarpNet, PaperEdge), DocRes, the Hertlein models (Inv3D GeoTr variants, DocMatcher) and UVDoc itself (original MIT, PaddleOCR re-release Apache-2.0) have permissive code licences with downloadable weights; the whole USTC family (DocTr, DocTr++, DocGeoNet, DocScanner) is under a custom non-commercial, share-alike "All Rights Reserved" licence that is incompatible with GPL-3.0 redistribution. Most other requested models (FDRNet, RDGR, Marior, LA-DocFlatten, DocReal, DocTLNet, Foldformer, UniDoc) have no usable released weights at all.

### Cited Findings

Summary table (GitHub licence field read through the GitHub repos API; LICENSE files opened where noted):

| Model | Code licence (verified) | Weights licence | GPL-3.0 embed OK? |
|---|---|---|---|
| UVDoc (baseline) | MIT (LICENSE opened) | In-repo `model/best_model.pkl`, so covered by repo MIT | Yes |
| PaddleOCR UVDoc | Apache-2.0 (HF card tag) | Apache-2.0 tag on the weights repos | Yes (Apache-2.0 is GPLv3-compatible) |
| DewarpNet | MIT (LICENSE opened, "Copyright (c) 2019 CVLab@StonyBrook") | No separate statement; weights on Google Drive linked from MIT repo | Probably (weights licence implicit) |
| PaperEdge | MIT (LICENSE opened, 2022 CVLab@StonyBrook) | No separate statement; Google Drive | Probably (implicit) |
| DocRes | MIT (LICENSE opened, 2024 Jiaxin Zhang) | No separate statement; OneDrive; HF mirrors tag MIT | Probably (implicit) |
| Inv3D models (GeoTr@doc3d, GeoTr@inv3d, GeoTr-template) | MIT (LICENSE opened, 2023 Felix Hertlein) | No separate statement; auto-downloaded by code | Unclear: training data is CC BY-NC-SA |
| DocMatcher | MIT (LICENSE opened, 2025 Felix Hertlein) | No separate statement | Unclear: trained on Inv3D (CC BY-NC-SA) |
| DocTr | Custom non-commercial licence | Same ("code, documentation, and data") | No |
| DocTr++ | Same custom licence | Same | No |
| DocGeoNet | Same custom licence | Same | No |
| DocScanner | Same custom licence | Same | No |
| DvD (diffusion, SIGGRAPH Asia 2025) | AGPL-3.0 | No separate statement; Google Drive | No for a GPL-3.0-only binary in practice (AGPL adds network clause; combination allowed only under GPLv3 s.13 terms) and far too heavy anyway |
| LA-DocFlatten | MIT (code) | No weights found | n/a |
| Marior | No licence file | No weights (results only) | n/a |
| DocReal | MIT file, but no source code | none | n/a |
| ForCenNet (ICCV 2025) | No LICENSE file | README is a TODO list | No (all rights reserved) |
| AADD (AAAI 2026), ArbDR (TMM 2026) | README badge says Apache-2.0 but no LICENSE file present | "full code and models will be released" | Not yet |
| BookNet (TMM 2026) | README says MIT; LICENSE file not found at main/master | Datasets explicitly non-commercial | Unclear |
| xiaomore/document-image-dewarping (ICCV 2023 FTA) | No LICENSE file | Google Drive/Baidu | No (all rights reserved) |
| gwxie displacement-flow (2020) | MIT | not checked | - |

- DewarpNet repo: MIT, 629 stars, last push 2024-11-10 — [GitHub API](https://api.github.com/repos/cvlab-stonybrook/DewarpNet), [LICENSE](https://raw.githubusercontent.com/cvlab-stonybrook/DewarpNet/master/LICENSE)
- PaperEdge repo: MIT — [LICENSE](https://raw.githubusercontent.com/cvlab-stonybrook/PaperEdge/main/LICENSE)
- DocRes repo: MIT, 691 stars, last push 2025-08-03 — [LICENSE](https://raw.githubusercontent.com/ZZZHANG-jx/DocRes/master/LICENSE)
- UVDoc repo: MIT, model shipped in the repo at `model/best_model.pkl` — [UVDoc README](https://github.com/tanguymagne/UVDoc)
- inv3d-model: MIT — [LICENSE](https://raw.githubusercontent.com/FelixHertlein/inv3d-model/main/LICENSE); doc-matcher: MIT — [LICENSE](https://raw.githubusercontent.com/FelixHertlein/doc-matcher/main/LICENSE)
- DocTr, DocTr-Plus, DocGeoNet, DocScanner all carry an identical custom `LICENSE.md` (GitHub reports `NOASSERTION`). Exact wording: "Copyright © Hao Feng 2024. All Rights Reserved."; "'Algorithm' refers to the deep learning algorithm contained in this repository, including all associated code, documentation, and data."; "'Non-Commercial Use' means use for academic research, personal study, or non-profit projects, without any direct or indirect commercial advantage."; "2.2 Commercial Use: Any Commercial Use of the Algorithm is strictly prohibited without explicit prior written permission from the Author."; "3.1 ... b) Share-Alike: If you modify, transform, or build upon the Algorithm, you must distribute your contributions under the same license as this one. c) No additional restrictions ..." — [DocTr LICENSE.md](https://raw.githubusercontent.com/fh2019ustc/DocTr/main/LICENSE.md), [DocTr-Plus](https://raw.githubusercontent.com/fh2019ustc/DocTr-Plus/master/LICENSE.md), [DocGeoNet](https://raw.githubusercontent.com/fh2019ustc/DocGeoNet/main/LICENSE.md), [DocScanner](https://raw.githubusercontent.com/fh2019ustc/DocScanner/main/LICENSE.md)
- Their READMEs add: "For commercial usage, please contact Professor Wengang Zhou (zhwg@ustc.edu.cn) and Hao Feng (haof@mail.ustc.edu.cn)." — [DocTr README](https://github.com/fh2019ustc/DocTr); same sentence with Hao Feng only in [DocTr-Plus](https://github.com/fh2019ustc/DocTr-Plus), [DocGeoNet](https://github.com/fh2019ustc/DocGeoNet), [DocScanner](https://github.com/fh2019ustc/DocScanner)
- A third-party Hugging Face mirror tags DocScanner-L weights as MIT (`shihabsikder092/DocScanner-L`, 29.3 MB `.pth`); this contradicts the upstream custom non-commercial licence and should not be relied on — [HF model](https://huggingface.co/shihabsikder092/DocScanner-L) vs [upstream LICENSE.md](https://raw.githubusercontent.com/fh2019ustc/DocScanner/main/LICENSE.md)
- Marior: repo has no licence (GitHub API licence = null) and the README only "provides the dewarping results for Marior ... on DocUNet dataset" — no code, no weights — [Marior](https://github.com/ZZZHANG-jx/Marior)
- DocReal: "Due to commercial restrictions, we are unable to provide the source code currently. Instead, we will provide a demo API in the future" — [DocReal README](https://github.com/irisXcoding/DocReal)
- LA-DocFlatten: MIT code licence; README covers a mesh dataset ("the dataset is subject to a different license than the code") and benchmark result images; no pretrained-weights link in the README — [LA-DocFlatten](https://github.com/BunnySoCrazy/LA-DocFlatten)
- FDRNet (CVPR 2022) has no code link, RDGR (CVPR 2022) only a Google Drive results link, in the curated list maintained by the DocTr authors; DocTLNet, Foldformer and UniDoc do not appear in that list at all — [Awesome-Document-Image-Rectification](https://github.com/fh2019ustc/Awesome-Document-Image-Rectification)
- DvD: AGPL-3.0 LICENSE file; pretrained weights on Google Drive — [DvD LICENSE](https://raw.githubusercontent.com/hanquansanren/DvD/master/LICENSE), [DvD README](https://github.com/hanquansanren/DvD)
- AADD: "[Coming Soon] The full code and models will be released after the final camera-ready submission."; licence badge Apache-2.0 but no LICENSE file retrievable on main/master — [AADD](https://github.com/chaoyunwang/AADD); ArbDR similar (results only, released 2026-02-27) — [ArbDR](https://github.com/chaoyunwang/ArbDR)
- BookNet: "The code in this repository is released under the MIT License ... the license covers the code only; the accompanying datasets (Book3D and Book100) are provided for non-commercial academic research use" — [BookNet](https://github.com/Liu-Shaokai/BookNet)
- PaddleOCR UVDoc weights on HF are tagged `license: apache-2.0` (Paddle format 32.1 MB, ONNX 31.7 MB, safetensors variant) — [PaddlePaddle/UVDoc](https://huggingface.co/PaddlePaddle/UVDoc), [PaddlePaddle/UVDoc_onnx](https://huggingface.co/PaddlePaddle/UVDoc_onnx)
- RapidUnDistort: README states Apache 2.0 (links to a LICENSE in a sibling repo); GitHub reports no licence file in the repo itself; its unwarp model is a converted UVDoc — [RapidUnDistort](https://github.com/RapidAI/RapidUnDistort)

### Inferences
- The USTC licence is effectively a home-made CC BY-NC-SA: it forbids commercial use and forbids "additional restrictions", while GPL-3.0 forbids imposing a non-commercial restriction on recipients. Embedding DocTr/DocTr++/DocGeoNet/DocScanner weights in a GPL-3.0 binary is therefore not possible without written permission from Hao Feng. The licence's definition of "Algorithm" explicitly includes "data", so weights are covered.
- For MIT repos whose weights sit on Google Drive/OneDrive with no separate statement (DewarpNet, PaperEdge, DocRes, Hertlein), the common reading is that the authors intended the weights to follow the repo licence; third parties already redistribute DewarpNet and DocRes weights under MIT on Hugging Face on that basis. That is an interpretation, not an explicit grant.
- Repos with no LICENSE file (Marior, ForCenNet, xiaomore FTA, currently AADD/ArbDR) are all-rights-reserved by default regardless of README badges.
- Realistic permissively licensed candidates with weights: DewarpNet, PaperEdge, DocRes (dewarping task), Inv3D GeoTr@doc3d, plus UVDoc re-trainings.

### Gaps
- DocTLNet, Foldformer, UniDoc: I found no repository or weights (one targeted search, plus absence from the curated list). Not exhaustively searched; treat as "no public weights found".
- FDRNet, RDGR: relied on the curated list; did not independently search for unofficial re-implementations.
- AADD/ArbDR/BookNet/ForCenNet: release status may change; BookNet weights hosting was not established.
- Whether authors of MIT-licensed repos consider Drive-hosted weights MIT was not confirmed with any author statement or issue thread.

## 2. Training-data licences and whether they bind weights

### Takeaway
Doc3D is now published as MIT (GitHub repo licence and Hugging Face dataset tag), which removes the usual worry for Doc3D-trained weights; UVDoc dataset repo is MIT; Inv3D is CC BY-NC-SA 4.0 (some files "academic use only"), which clouds Inv3D-trained models. Whether dataset NC terms bind weights is legally unsettled, with common practice being to carry NC terms over.

### Cited Findings
- Doc3D: the `doc3D-dataset` GitHub repo has an MIT LICENSE ("Copyright (c) 2020 CVLab@StonyBrook"); since 2025-12-18 the data is on Hugging Face ("No need to fill the Google Form anymore"; meshes still by e-mail request). Textures came from "Yes! Magazine under Creative Commons Licence", CVF Open Access and Project Gutenberg — [doc3D-dataset README](https://github.com/cvlab-stonybrook/doc3D-dataset), [LICENSE](https://raw.githubusercontent.com/cvlab-stonybrook/doc3D-dataset/master/LICENSE)
- Doc3D on Hugging Face: licence tag `mit`; gated ("You need to agree to share your Hugging Face information to access this dataset"); no non-commercial wording visible on the public page — [HF dataset](https://huggingface.co/datasets/StonyBrook-CVLab/doc3D-dataset)
- UVDoc dataset: repo licence MIT ("Copyright (c) 2023 Tanguy MAGNE"); data hosted as zip files at igl.ethz.ch — [UVDoc-Dataset](https://github.com/tanguymagne/UVDoc-Dataset), [LICENSE](https://raw.githubusercontent.com/tanguymagne/UVDoc-Dataset/main/LICENSE)
- UVDoc model was trained on both the UVDoc dataset and Doc3D (training instructions list both) — [UVDoc README](https://github.com/tanguymagne/UVDoc)
- Inv3D dataset: "CC BY-NC-SA 4.0", with files listed in `restricted-license-files.txt` for academic use only — [KIT repository record](https://publikationen.bibliothek.kit.edu/1000161884)
- Inv3D model zoo: `geotr@doc3d`, `geotr@inv3d`, `geotr_template@inv3d`, `geotr_template_large@inv3d` (the template models need a flat template image as extra input) — [inv3d-model README](https://github.com/FelixHertlein/inv3d-model); DocMatcher zoo: `dewarpnet@inv3d`, `geotr@inv3d`, `geotr_template@inv3d`, `geotr_template_large@inv3d`, `docmatcher@inv3d` — [doc-matcher README](https://github.com/FelixHertlein/doc-matcher)
- DewarpNet trained on Doc3D (weight files named `unetnc_doc3d.pkl`, `dnetccnl_doc3d.pkl`) — [DewarpNet README](https://github.com/cvlab-stonybrook/DewarpNet)
- PaperEdge: "The code and the DIW dataset for 'Learning From Documents in the Wild to Improve Document Unwarping'"; DIW (2.13 GB) is a Google Drive link in the MIT repo with no separate dataset licence text in the README — [PaperEdge README](https://github.com/cvlab-stonybrook/PaperEdge)
- xiaomore FTA model: "We use the Doc3D dataset for training" — [repo](https://github.com/xiaomore/document-image-dewarping)
- DIR300 test set is distributed as a Google Drive folder from the DocGeoNet repo, i.e. under that repo's non-commercial custom licence ("code, documentation, and data") — [DocGeoNet README](https://github.com/fh2019ustc/DocGeoNet), [LICENSE.md](https://raw.githubusercontent.com/fh2019ustc/DocGeoNet/main/LICENSE.md)
- BookNet datasets: "released for non-commercial scientific research purposes only" — [BookNet](https://github.com/Liu-Shaokai/BookNet)
- Legal status: The Turing Way notes it is an open question whether model weights are even copyrightable — [The Turing Way](https://book.the-turing-way.org/reproducible-research/licensing/licensing-ml); a scholarly analysis argues models are plausibly derivative works of training data so ND/NC-SA terms are the riskiest (authors' analysis, not a holding) — [Lee, Cooper, Grimmelmann, arXiv 2309.08133](https://arxiv.org/pdf/2309.08133); practitioner view on HF forums is that a model can be licensed independently of its datasets — [HF forum thread](https://discuss.huggingface.co/t/license-for-philschmid-bart-large-cnn-samsum/25695); common cautious practice is to carry NC terms to weights, e.g. — [OPR-Project model card](https://huggingface.co/OPR-Project/Registration-nuScenes/blob/main/README.md)

### Inferences
- Doc3D-only models from MIT repos (DewarpNet, DocRes dewarping, GeoTr@doc3d from inv3d-model) are the cleanest. Doc3D historically required a Google-Form request, and many secondary sources still describe it as research-only; the currently published licence is MIT. I could not see the gated HF access text, so a residual condition there cannot be excluded.
- Models trained on Inv3D (all DocMatcher zoo entries, `*@inv3d`) carry NC-SA risk even though the code is MIT; cautious projects would not embed them in a GPL-3.0 binary.
- Benchmarks (DocUNet, DIR300) are only used for evaluation by these models, so their terms do not affect weights.

### Gaps
- DocUNet benchmark licence: not checked (Stony Brook page not opened); no licence text known.
- DIW licence: no explicit text found beyond being shipped from an MIT repo.
- Exact training sets of DocRes (dataset instruction file `data/README.md` not opened), DocTr++, DocScanner, DocGeoNet were not verified this session (from prior knowledge: DocTr/DocGeoNet/DocScanner on Doc3D; DocTr++ on a Doc3D-derived unrestricted set; DocRes dewarping on Doc3D).
- The gated-access conditions of the Doc3D HF dataset could not be read without login.

## 3. Where weights are hosted and whether they are downloadable

### Takeaway
Most research weights are on Google Drive / Baidu / OneDrive links in READMEs; only UVDoc (in-repo), PaddleOCR UVDoc, and third-party mirrors of DewarpNet/DocRes/DocScanner are on durable hosts (GitHub/Hugging Face). I verified HF file listings via the HF API but did not attempt Drive/Baidu/OneDrive downloads.

### Cited Findings
- UVDoc: in-repo `model/best_model.pkl` — [UVDoc](https://github.com/tanguymagne/UVDoc)
- PaddleOCR UVDoc: HF `PaddlePaddle/UVDoc` (`inference.pdiparams` 32.1 MB, ~461k downloads), `PaddlePaddle/UVDoc_onnx` (`inference.onnx` 31.7 MB, last modified 2026-06-10), `PaddlePaddle/UVDoc_safetensors`; not gated — [HF API](https://huggingface.co/api/models/PaddlePaddle/UVDoc_onnx), [UVDoc](https://huggingface.co/PaddlePaddle/UVDoc)
- DewarpNet: Google Drive (pre-trained and "final models" folders) — [DewarpNet README](https://github.com/cvlab-stonybrook/DewarpNet); HF mirrors: `litert-community/DewarpNet-LiteRT` (`dewarp.tflite` 188.9 MB, MIT tag) — [HF](https://huggingface.co/litert-community/DewarpNet-LiteRT); `mlboydaisuke/DewarpNet-OpenVINO` (`dewarp.onnx` 189.3 MB, MIT tag, modified 2026-11-07) — [HF](https://huggingface.co/mlboydaisuke/DewarpNet-OpenVINO)
- PaperEdge: Google Drive, "Pretrained models (139.7MB each)" Enet + Tnet — [PaperEdge README](https://github.com/cvlab-stonybrook/PaperEdge)
- DocRes: OneDrive (`1drv.ms`) for `docres.pkl` and `mbd.pkl`; HF mirrors `DaVinciCode/doctra-docres-main` (`docres.pkl` 183.3 MB, MIT, "Weights are redistributed under the same terms") and `presencesw/DocRes` (`docres.pkl` 183.3 MB, `mbd.pkl` 713.0 MB) — [DocRes README](https://github.com/ZZZHANG-jx/DocRes), [doctra mirror](https://huggingface.co/DaVinciCode/doctra-docres-main), [presencesw mirror](https://huggingface.co/presencesw/DocRes)
- DocTr: Google Drive or Baidu Cloud (pwd h4az) — [DocTr README](https://github.com/fh2019ustc/DocTr); DocGeoNet: Google Drive — [README](https://github.com/fh2019ustc/DocGeoNet); DocScanner: Google Drive "pre-trained DocScanner-L" — [README](https://github.com/fh2019ustc/DocScanner)
- DocTr++: README says "Put the pretrained model to `$ROOT/model_pretrained/`" with no download link in the lines I extracted — [DocTr-Plus README](https://github.com/fh2019ustc/DocTr-Plus)
- Inv3D / DocMatcher: "The models will be downloaded automatically before the inference starts." — [inv3d-model README](https://github.com/FelixHertlein/inv3d-model)
- DvD: Google Drive checkpoints; HF Space demo — [DvD README](https://github.com/hanquansanren/DvD)
- YomiToku (Japanese OCR project) published dewarper weights on HF in Nov 2026: `KotaroKinoshita/yomitoku-dewarper-uvdoc-v1` (32.8 MB safetensors) and `yomitoku-dewarper-pagenet-v1` (43.9 MB); no licence tag on either — [HF uvdoc](https://huggingface.co/KotaroKinoshita/yomitoku-dewarper-uvdoc-v1), [HF pagenet](https://huggingface.co/KotaroKinoshita/yomitoku-dewarper-pagenet-v1)

### Inferences
- Drive/Baidu/OneDrive links for 2019-2023 papers are the fragile part; HF mirrors of DewarpNet and DocRes mean those two are practically obtainable today even if the original links rot.
- The YomiToku weights have no licence tag; YomiToku itself has historically been CC BY-NC-SA (prior knowledge, not verified), so assume not usable.

### Gaps
- I did not test-download any Google Drive, Baidu or OneDrive link, so "downloadable today" is unverified for DewarpNet/PaperEdge/DocRes originals and all USTC models.
- DocTr++ weight location not found; Inv3D download URL host not identified.

## 4. Existing ONNX exports and export difficulty

### Takeaway
Ready-made ONNX exists only for UVDoc (several, including a grid-output variant and Paddle's official one) and DewarpNet (189 MB, converted TFLite to ONNX, opset 17). Nothing was found for DocTr, DocTr++, DocGeoNet, DocScanner, PaperEdge, DocRes or the Hertlein models; those need self-export and involve attention, GridSample and/or unrolled recurrent refinement.

### Cited Findings
- `PaddlePaddle/UVDoc_onnx`: official ONNX, 31.7 MB, Apache-2.0; usable through `paddleocr text_image_unwarping ... --engine onnxruntime` — [HF](https://huggingface.co/PaddlePaddle/UVDoc_onnx)
- `fredcallagan/uvdoc-grid-onnx`: UVDoc export with grid output, opset 16, external data file 31.6 MB, Apache-2.0 tag; card notes "Unlike the PaddlePaddle ONNX variant that outputs a fixed 288x288 image, this version" outputs the grid — [HF](https://huggingface.co/fredcallagan/uvdoc-grid-onnx)
- Other UVDoc ONNX: `matrixmaven/uvdoc` (`uvdoc.onnx` 16.1 MB plus separate `bilinear_unwarping.onnx`, MIT tag; size suggests fp16), `twmht/UVDoc_infer_onnx`, `Shivin11/occular-uvdoc` — [HF search API](https://huggingface.co/api/models?search=uvdoc), [matrixmaven/uvdoc](https://huggingface.co/matrixmaven/uvdoc)
- RapidUnDistort: "转换 UVDoc 模型为onnx" (converted UVDoc to ONNX); its unwrap stage is UVDoc only (the other stages are deshadow/deblur/binarize) — [RapidUnDistort README](https://github.com/RapidAI/RapidUnDistort)
- DewarpNet ONNX: `mlboydaisuke/DewarpNet-OpenVINO` `dewarp.onnx` 189,339,174 bytes, produced "as ONNX (tf2onnx 1.17.0, opset 17)" from the LiteRT conversion; the LiteRT model is a merged two-CNN graph of 371 ops and the unwarp step is host-side ("blur the map (3×3), resize to the original image size, then `grid_sample(original_image, map)`") — [HF OpenVINO card](https://huggingface.co/mlboydaisuke/DewarpNet-OpenVINO), [HF LiteRT card](https://huggingface.co/litert-community/DewarpNet-LiteRT)
- HF model search returned nothing relevant for `unwarp`, `docgeonet`, `paperedge`, `document-unwarping`; `doctr` hits are the unrelated Mindee docTR OCR library; `docres` hits are PyTorch `.pkl` mirrors only; `docscanner` only a PyTorch `.pth` mirror — [HF search API](https://huggingface.co/api/models?search=docscanner)
- The DocTr repo ships Hugging Face Space demos (PyTorch), not ONNX — [DocTr README](https://github.com/fh2019ustc/DocTr)

### Inferences
(Architecture op requirements below are from prior knowledge of the papers/code, not re-verified this session.)
- DewarpNet: pure CNN (UNet shape network + DenseNet-style backward-map network): Conv, ConvTranspose, BatchNorm, ReLU/LeakyReLU, Tanh/Hardtanh(Clip), Concat. No attention, no GridSample inside the net. Easiest non-UVDoc export; the existing tf2onnx graph will contain NHWC Transpose patterns, so a fresh `torch.onnx.export` from the MIT PyTorch weights is the cleaner route for tract.
- PaperEdge: two CNN encoder-decoders (Enet, Tnet) with a GridSample warp between them and bilinear interpolation of the predicted grid: Conv, norm, ReLU, Resize(bilinear, align_corners), GridSample. Can also be exported as two graphs with the warp done in Rust (the tool already does grid warping for UVDoc).
- DocRes: Restormer-style transformer (channel-wise "transposed" attention, depthwise convs, GELU, LayerNorm, PixelShuffle/PixelUnshuffle = DepthToSpace/SpaceToDepth); dewarping task additionally needs the separate MBD segmentation model (713 MB mirror file) to build its prompt, and outputs a dense backward map. Exportable in principle (MatMul, Softmax, Div/L2-norm, Erf or Gelu, DepthToSpace) but heavy.
- DocTr / Inv3D GeoTr: transformer encoder-decoder with `nn.MultiheadAttention`, learned queries, LayerNorm, plus RAFT-style convex upsampling (Unfold + Softmax) and a segmentation pre-net with bilinear Resize. Exports decompose attention into MatMul/Softmax/Reshape/Transpose; Unfold exports as slices/gathers or needs rewriting.
- DocTr++: same family, hierarchical; no segmentation pre-net.
- DocGeoNet / DocScanner: RAFT-derived ConvGRU iterative refinement with a fixed iteration count (unrolls statically into a long Conv/Sigmoid/Tanh graph, no ONNX Loop needed) plus GridSample-style sampling in the loop (DocScanner) and attention blocks (DocGeoNet).
- DocMatcher: multi-stage pipeline (line detection, LightGlue-style matcher, template input), not a single exportable graph; also requires a template image, so unsuitable for a generic scanner tool.
- DvD: diffusion with many denoising steps; out of scope for CPU in tens of seconds.

### Gaps
- No evidence found of community ONNX exports for DocTr/DocTr++/DocGeoNet/DocScanner/PaperEdge/DocRes; absence is based on HF search only (no ModelScope, no GitHub code search, which required authentication).
- I did not download and inspect the op set of any ONNX file (e.g. the DewarpNet ONNX) to confirm tract loadability.
- paddle2onnx specifics for UVDoc not checked; moot since an official ONNX is published.

## 5. tract operator support (tract-onnx 0.21 to 0.23)

### Takeaway
tract 0.23.x (the project pins tract-onnx 0.23.10, released 2026-11-05) registers every operator the candidate architectures need, including GridSample (opset 16+, new in 0.23), Einsum, LayerNormalization, Gelu (new in 0.23), Erf, MatMul, Softmax, Resize (opsets 10 to 18+), ScatterND, InstanceNormalization, ConvTranspose, If and Scan; the notable absence is ONNX `Loop`. On 0.21/0.22, GridSample and the Gelu op are missing.

### Cited Findings
- Project lock file pins `tract-onnx` 0.23.10 — local `/home/phillip/Code/auto-crop/Cargo.lock`; latest releases: v0.23.10 (2026-11-05), v0.23.9 (2026-10-15), 0.22.5 (2026-10-15) — [tract releases API](https://api.github.com/repos/sonos/tract/releases?per_page=3)
- Top-level ONNX registrations on main: "Constant", "Einsum", "Identity", "GridSample", "Resize", "NonMaxSuppression", "Multinomial" — [onnx/src/ops/mod.rs](https://raw.githubusercontent.com/sonos/tract/main/onnx/src/ops/mod.rs)
- NN ops registered: ArgMax/ArgMin, AveragePool, BatchNormalization, Conv, ConvInteger, ConvTranspose, Dropout, Elu, Global*Pool, GroupNormalization, HardSigmoid, InstanceNormalization, LayerNormalization, LeakyRelu, LpNormalization, LogSoftmax, LRN, MaxPool, MeanVarianceNormalization, PRelu, Reduce* (L1, L2, LogSum, LogSumExp, Max, Mean, Min, Prod, Sum, SumSquare), Relu, Selu, Sigmoid, Softmax, Softplus, Softsign, HardSwish, Mish, Gelu, plus ORT-contrib ops Attention, MultiHeadAttention, GroupQueryAttention, BiasGelu, FastGelu, QuickGelu, RMSNormalization, RotaryEmbedding, SimplifiedLayerNormalization, SkipLayerNormalization — [onnx/src/ops/nn/mod.rs](https://raw.githubusercontent.com/sonos/tract/main/onnx/src/ops/nn/mod.rs)
- Array ops registered: Compress, Concat, ConstantOfShape, Expand, Flatten, Gather, GatherElements, GatherND, NonZero, OneHot, Range, Pad, Reshape, Scatter, ScatterElements, ScatterND, Shape, Size, Slice, Split, Squeeze, Tile, TopK, Transpose, Trilu, Unsqueeze — [onnx/src/ops/array/mod.rs](https://raw.githubusercontent.com/sonos/tract/main/onnx/src/ops/array/mod.rs)
- Math ops registered include Add, Sub, Mul, Div, Pow, Sqrt, Rsqrt, Exp, Log, Erf, Tanh, Clip, Max, Min, MatMul, MatMulInteger, QLinearMatMul, Gemm — [onnx/src/ops/math.rs](https://raw.githubusercontent.com/sonos/tract/main/onnx/src/ops/math.rs)
- Logic/control: Where and "If" registered — [logic.rs](https://raw.githubusercontent.com/sonos/tract/main/onnx/src/ops/logic.rs); recurrent: "GRU", "LSTM", "RNN", "Scan" — [rec.rs](https://raw.githubusercontent.com/sonos/tract/main/onnx/src/ops/rec.rs). No `"Loop"` registration was found in mod.rs, rec.rs, logic.rs or rec/scan.rs.
- GridSample: modes bilinear/linear, nearest, bicubic/cubic; padding zeros, border, reflection; `align_corners` honoured; requires ONNX opset 16 or later ("Unsupported operator set for GridSample operator" otherwise); rank-generic (spatial rank = rank - 2) — [grid_sample.rs](https://raw.githubusercontent.com/sonos/tract/main/onnx/src/ops/grid_sample.rs)
- Resize: dispatches on opset 10, 11-12, 13-17, 18+; reads `coordinate_transformation_mode` (default half_pixel), `mode` (default nearest), `nearest_mode`, `cubic_coeff_a`; errors on "Resize with neither scales nor sizes" — [resize.rs](https://raw.githubusercontent.com/sonos/tract/main/onnx/src/ops/resize.rs)
- Version check by file existence per tag: `grid_sample.rs`, `nn/gelu.rs`, `nn/multi_head_attention.rs` exist at v0.23.0 and v0.23.10 but return 404 at 0.22.5 and 0.21.13; `einsum.rs`, `nn/layer_norm.rs`, `nn/instance_norm.rs`, `resize.rs` exist in all four; 0.21.13 and 0.22.5 `nn/mod.rs` register ConvTranspose, InstanceNormalization, LayerNormalization, Softmax — [v0.23.10 grid_sample.rs](https://raw.githubusercontent.com/sonos/tract/v0.23.10/onnx/src/ops/grid_sample.rs), [0.22.5 nn/mod.rs](https://raw.githubusercontent.com/sonos/tract/0.22.5/onnx/src/ops/nn/mod.rs)
- tract README: ONNX is load-only; recommended deployment is converting once to NNEF/tract-OPL (`tract model.onnx dump --nnef model.nnef.tgz`) and shipping `tract-core` + `tract-nnef` (+ `tract-onnx-opl` for ONNX-only operators); PyTorch can also be exported directly with `torch-to-nnef`; symbolic shapes are documented in `doc/symbolic-shapes.md` — [tract README](https://github.com/sonos/tract/blob/main/README.md), [doc dir](https://github.com/sonos/tract/tree/main/doc)

### Inferences
- With tract 0.23.10, operator coverage is not the blocker for any candidate: CNN models (DewarpNet), GridSample models (PaperEdge, DocScanner) and transformer models (DocTr, GeoTr, DocRes) all map to registered ops, provided export uses opset 16 or later (17 recommended so LayerNormalization is emitted as one node) and fixed input shapes.
- Risks that only a load test reveals: (a) `torch.onnx.export` of `nn.MultiheadAttention` and `Unfold` yields Shape/Gather/ConstantOfShape-driven dynamic reshapes that tract's type inference must fold; export with static shapes and run onnx-simplifier first. (b) ONNX `Loop` is not supported, so iterative models must be exported with the loop unrolled (default tracing behaviour for a Python `for` with fixed count). (c) Operator registration says nothing about speed; tract's GridSample/attention kernels are much less tuned than its Conv path.
- The Gelu op only matters for opset 20 exports; lower opsets emit Erf-based GELU, which works on 0.21+.
- Because the tool already performs the final grid warp in Rust, GridSample is only needed for models that sample inside the network.

### Gaps
- I did not open an official tract operator-coverage document or ONNX backend-test pass list, nor tract issues about dynamic shapes or attention; statements are from source registration only.
- No candidate model was actually loaded in tract; compatibility is predicted, not tested.
- GridSample numerical parity with PyTorch `align_corners` semantics in tract was not tested.

## 6. Model sizes and CPU latency

### Takeaway
Only UVDoc (about 30 MB, sub-second CPU) and DewarpNet (about 189 MB, 0.2 to 0.6 s CPU) have published CPU numbers; the others are in the 30 to 280 MB range on disk with no CPU latency reports found, and the transformer/iterative ones will be several times slower in tract than UVDoc.

### Cited Findings
- PaddleOCR UVDoc: 30.3 MB; GPU 19.05 ms (Tesla T4); CPU high-performance mode 869.82 ms on Intel Xeon Gold 6271C @ 2.60GHz, FP32, 8 threads (normal-mode CPU not reported) — [PaddleOCR text image unwarping docs](http://www.paddleocr.ai/latest/en/version3.x/module_usage/text_image_unwarping.html)
- DewarpNet ONNX (189.3 MB): 201.36 ms median on Core Ultra 9 386H CPU with ONNX Runtime 1.31.1 CPU EP — [HF card](https://huggingface.co/mlboydaisuke/DewarpNet-OpenVINO); LiteRT version 594 ms p50 on Raspberry Pi 5, XNNPACK, 4 threads — [HF card](https://huggingface.co/litert-community/DewarpNet-LiteRT)
- PaperEdge: two checkpoints of 139.7 MB each (about 280 MB total) — [PaperEdge README](https://github.com/cvlab-stonybrook/PaperEdge)
- DocRes: `docres.pkl` 183.3 MB; dewarping also needs `mbd.pkl` 713.0 MB (mirror file sizes; checkpoints may include optimizer state) — [HF mirror listing](https://huggingface.co/presencesw/DocRes)
- DocScanner-L: 29.3 MB `.pth` (mirror) — [HF](https://huggingface.co/shihabsikder092/DocScanner-L)
- UVDoc ONNX: 31.6 to 31.7 MB fp32; a 16.1 MB variant exists — [fredcallagan](https://huggingface.co/fredcallagan/uvdoc-grid-onnx), [matrixmaven](https://huggingface.co/matrixmaven/uvdoc)

### Inferences
- DewarpNet at about 0.2 s under ONNX Runtime suggests low single-digit seconds at worst in tract; comfortably inside the "tens of seconds" budget, but its 189 MB would grow the embedded binary about sixfold relative to UVDoc.
- PaperEdge (about 280 MB) and DocRes+MBD (about 900 MB as checkpoints) are impractical to commit to a GitHub repo and embed, independent of speed (GitHub blocks files over 100 MB without LFS).
- DocScanner-L is small (29 MB) but licence-blocked; its iterative refinement multiplies compute.

### Gaps
- No CPU latency figures found for DocTr, DocTr++, DocGeoNet, DocScanner, PaperEdge, DocRes, GeoTr@doc3d; DocTr/DocGeoNet/GeoTr file sizes were not verified (not on HF).
- No tract-specific timing for any model other than what the tool's author already has for UVDoc.

## 7. Other pure-Rust inference options

### Takeaway
If tract fails on a model, the pure-Rust fallbacks are Burn's ONNX import (code generation), Candle (hand-written model or `candle-onnx`), and RTen; wonnx is GPU/WebGPU-oriented and `ort` is a C++ ONNX Runtime binding (excluded). None were verified in this session.

### Cited Findings
- tract itself offers a second route that avoids ONNX entirely: PyTorch to NNEF via `torch-to-nnef`, "maintained alongside tract" — [tract README](https://github.com/sonos/tract/blob/main/README.md), [torch-to-nnef](https://github.com/sonos/torch-to-nnef)
- RTen is discussed as a pure-Rust ONNX-derived runtime where models "will already work if the exporter created an ONNX graph using the constituent operations" (search-result snippet of an rten issue via a mirror; primary not opened) — [rten issue 14 mirror](https://github.hscsec.cn/robertknight/rten/issues/14)

### Inferences
- From prior knowledge (unverified this session): Burn (`burn-import`/`burn-onnx`, https://github.com/tracel-ai/burn) converts ONNX to Rust source plus a weights file at build time with an `ndarray` CPU backend, and its op coverage is narrower than tract's; Candle (https://github.com/huggingface/candle) is pure Rust with CPU backend and an incomplete `candle-onnx` crate, so it realistically means re-implementing the model by hand and loading safetensors; RTen (https://github.com/robertknight/rten) is a pure-Rust runtime with its own model format converted from ONNX, built for OCR-type CNN/transformer models; wonnx (https://github.com/webonnx/wonnx) runs on wgpu (GPU) and has seen little maintenance, so it does not fit a no-GPU static musl binary.
- Given tract 0.23 already registers GridSample, attention and normalisation ops, switching runtimes is unlikely to be necessary; the binding constraints are licences and model size, not operators.

### Gaps
- Burn/Candle/RTen/wonnx capabilities, licences and current maintenance state were not checked in this session; treat the above as leads only.

## Overall shortlist (inference across sections)

### Takeaway
Under the four hard constraints, only DewarpNet (MIT code, Doc3D now MIT, ONNX already exists, pure CNN, about 0.2 s CPU, but 189 MB) and alternative UVDoc weights (PaddleOCR Apache-2.0 re-release) are clearly deployable today; GeoTr@doc3d from inv3d-model (MIT, Doc3D-trained DocTr-style geometry net) is the one transformer candidate with a clean-looking licence, but needs self-export and a tract load test. PaperEdge and DocRes are licence-clean but too large; everything from USTC is licence-blocked; the rest have no weights.

### Cited Findings
- See sections 1 to 6; key supports: [DewarpNet LICENSE](https://raw.githubusercontent.com/cvlab-stonybrook/DewarpNet/master/LICENSE), [Doc3D LICENSE](https://raw.githubusercontent.com/cvlab-stonybrook/doc3D-dataset/master/LICENSE), [DewarpNet ONNX](https://huggingface.co/mlboydaisuke/DewarpNet-OpenVINO), [inv3d-model](https://github.com/FelixHertlein/inv3d-model), [USTC LICENSE.md](https://raw.githubusercontent.com/fh2019ustc/DocTr/main/LICENSE.md), [tract grid_sample.rs](https://raw.githubusercontent.com/sonos/tract/v0.23.10/onnx/src/ops/grid_sample.rs)

### Inferences
- Retraining the MIT UVDoc architecture (or a DocTr-like architecture re-implemented from the paper) on MIT-licensed data (Doc3D, UVDoc dataset) is the only path to a fold-specialised model with fully clean licensing if existing weights are inadequate.

### Gaps
- Whether `geotr@doc3d` weights are still auto-downloadable and their size was not verified.
