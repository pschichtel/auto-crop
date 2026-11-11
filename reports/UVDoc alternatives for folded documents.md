# Swapping UVDoc will not straighten creases

No dewarping model you can ship today in a GPL-3.0, tract-based, CPU-only tool will remove the crease kinks, and most would be a step backwards from UVDoc. Every neural dewarper from 2019 to 2026 predicts a low-resolution warp (a 9x9 to 45x31 control grid, or a flow regressed on a 36x36 to 64x64 lattice) and upsamples it smoothly, so a fold, which is a slope break only a few pixels wide in a 5000x7000 photo, gets smeared over at least one grid cell. The models that beat UVDoc on paper are unreleased (ForCenNet, AADD, ArbDR), licence-blocked (the DocTr / DocGeoNet / DocScanner family, DvD), or have no confirmed weights (SyntheticDoc's 121x177-grid baseline). The option most likely to fix the kinks is a classical post-step on top of UVDoc: trace the drawing's long lines and border frame, detect the creases, and solve a small least-squares grid refinement that is allowed to bend sharply at the creases. That design is assembled from published ingredients but has not been demonstrated on folded plans by anyone, so treat it as a well-founded experiment. The cheapest remedy is at capture: flatten the sheet under glass, shoot from further away and centre the camera. On the neural side, the one lead worth a quick check is whether the UVDoc authors' SyntheticDoc project has published weights for its high-resolution model.

## Every neural dewarper smooths a crease over at least one grid cell

UVDoc predicts a 45x31 backward-map grid from a 488x712 input and bilinearly interpolates it to the photo size ([UVDoc paper](https://arxiv.org/abs/2302.02887)). On a 5000x7000 photo that is one node every 160 px or so, roughly 13 to 19 mm on an A2 sheet. A slope break narrower than one cell cannot be represented, so the residual kink is a resolution limit of the representation and not a defect in UVDoc's training.

The alternatives are no finer. The table gives each model's working resolution and what it outputs.

| Model | Input | Output representation | Source |
|---|---|---|---|
| UVDoc | 488x712 | 45x31 grid, bilinear upsampling | [UVDoc paper](https://arxiv.org/abs/2302.02887) |
| DewarpNet | 256x256 | 128x128 backward map, 3x3 box blur, then resize | [infer.py](https://github.com/cvlab-stonybrook/DewarpNet/blob/master/infer.py) |
| DocTr, DocTr++ | 288x288 | flow regressed at 1/8 scale, upsampled to 288x288, then resized | [DocTr](https://arxiv.org/abs/2110.12942), [DocTr++](https://arxiv.org/abs/2304.08796) |
| DocScanner | 288x288 (36x36 features) | iterative flow at 1/8 scale, learned upsampling | [DocScanner](https://arxiv.org/abs/2110.14968) |
| PaperEdge | 256x256 | 64x64 deformation field, plus a second local field | [PaperEdge](https://zhixinshu.github.io/pdf/SIGGRAPH_22_PAPEREDGE.pdf) |
| FDRNet | not stated | 9x9 thin-plate-spline mesh | [FDRNet](https://openaccess.thecvf.com/content/CVPR2022/papers/Xue_Fourier_Document_Restoration_for_Robust_Document_Dewarping_and_Recognition_CVPR_2022_paper.pdf) |
| DDCP, DocReal | 992x992 | 31x31 control points | [DDCP](https://arxiv.org/abs/2203.10543), [DocReal](https://openaccess.thecvf.com/content/WACV2024/papers/Yu_DocReal_Robust_Document_Dewarping_of_Real-Life_Images_via_Attention-Enhanced_Control_WACV_2024_paper.pdf) |
| AADD, ArbDR | 712x488 | 45x31 grid (ArbDR composes several) | [AADD](https://arxiv.org/html/2507.15000), [ArbDR](https://arxiv.org/html/2511.23150) |
| DvD | 64x64 latent | diffusion over a coordinate map, upsampled | [DvD](https://arxiv.org/html/2505.21975v2) |
| ForCenNet | 288x288 | backward map from 1/8-scale features | [ForCenNet](https://arxiv.org/html/2507.19804v1) |
| SyntheticDoc baseline | 512x720 or 1024x1440 | 31x45 or **121x177** grid | [SyntheticDoc](https://arxiv.org/html/2609.15503v1) |

All of these output coordinates, so all can be applied to a 5000x7000 original the way UVDoc is. The exceptions are image-space diffusion restorers such as Uni-DocDiff, which appear to generate pixels (inferred from titles and abstracts, not verified) and should be avoided for a drawing. Learned upsampling does not rescue the dense-flow family: DocGeoNet's own ablation shows learnable versus bilinear upsampling changes local distortion (LD) only from 7.77 to 7.71 ([DocGeoNet](https://arxiv.org/abs/2210.08161)).

Training data points the same way. Doc3D, the dataset behind almost every 2019-2023 model, was captured with a depth sensor that, in its authors' words, "cannot capture fine details of deformation like subtle creases" ([DewarpNet](https://openaccess.thecvf.com/content_ICCV_2019/papers/Das_DewarpNet_Single-Image_Document_Unwarping_With_Stacked_3D_and_2D_Regression_ICCV_2019_paper.pdf)). UVDoc is the exception in that period: it was trained on physically folded paper with unsmoothed ground truth, and it has the best line-straightness scores on its own benchmark (1.82 / 2.48 horizontal / vertical, against 2.24 / 3.91 for DocGeoNet) ([UVDoc paper](https://arxiv.org/abs/2302.02887)). That benchmark comes from UVDoc's own capture pipeline, so some of the lead is home advantage, but it is the only published straight-line comparison.

No paper found reports a fold-only comparison that includes UVDoc, a crease-kink metric, or any evaluation on large-format technical drawings. Where authors discuss folds, they concede the weakness. ArbDR states that severe folds are not fully flattened and blames scarce high-frequency training data and single-view ambiguity ([ArbDR](https://arxiv.org/html/2511.23150)). PaperEdge "failed to handle some complex crumpled paper cases" ([PaperEdge](https://zhixinshu.github.io/pdf/SIGGRAPH_22_PAPEREDGE.pdf)). ForCenNet claims to handle "folds, creases, and shadows", but only qualitatively and with no released code to check ([ForCenNet](https://arxiv.org/html/2507.19804v1)).

## Newer models beat UVDoc on benchmarks, but the best ones cannot be shipped

On page-averaged geometric metrics, several 2025-2026 models are clearly ahead of UVDoc. These metrics are computed on downscaled A4-style text documents and barely register a localised kink, so they show general quality, not crease handling. Lower LD and AD (aligned distortion) are better.

| Model | DocUNet LD / AD | DIR300 LD / AD | Params | Weights public? | Licence |
|---|---|---|---|---|---|
| UVDoc (baseline) | 6.83 / 0.315 | 7.73 / 0.218 | 8M | yes, in repo | MIT; Paddle re-release Apache-2.0 |
| ForCenNet | 4.82 / 0.19 | 4.65 / 0.123 | not reported | no | no LICENSE file |
| SyntheticDoc 121x177 | 5.51 / 0.219 | not reported | UVDoc-derived | not confirmed | MIT label on code |
| SyntheticDoc 31x45 | 5.98 / 0.252 | not reported | UVDoc-derived | not confirmed | MIT label on code |
| ArbDR | 6.198 / 0.260 | not reported | 24.0M | no, "released soon" | Apache-2.0 badge, no LICENSE file |
| AADD | 6.249 / 0.278 | 4.261 / 0.131 | UVDoc architecture | no, "coming soon" | Apache-2.0 badge, no LICENSE file |
| DvD | 6.61 / 0.279 | not retrieved | 151.25M | yes | AGPL-3.0 |
| TADoc | 7.11 / 0.334 | 4.33 / 0.170 | 7.9M | no | none |
| DocScanner-L | 7.45 / n.a. | 7.06 / 0.225 | 8.5M | yes | custom non-commercial |
| D2Dewarp | 7.71 / 0.349 | 5.73 / 0.186 | 71.2M | yes | not checked |
| DocRes | 9.371 / 0.472 | 6.83 / 0.241 | 15.2M | yes | MIT |
| DewarpNet | 8.38 / 0.396 | 13.94 / 0.332 | 86.9M | yes, plus ONNX | MIT |

Sources: [UVDoc paper](https://arxiv.org/abs/2302.02887), [AADD](https://arxiv.org/html/2507.15000), [ArbDR](https://arxiv.org/html/2511.23150), [SyntheticDoc](https://arxiv.org/html/2609.15503v1), [ForCenNet](https://arxiv.org/html/2507.19804v1), [TADoc](https://arxiv.org/html/2508.06988), [DvD](https://arxiv.org/html/2505.21975v2), [D2Dewarp](https://arxiv.org/html/2507.08492), [DocScanner repo](https://github.com/fh2019ustc/DocScanner), [DocGeoNet](https://arxiv.org/abs/2210.08161), [FTA](https://openaccess.thecvf.com/content/ICCV2023/papers/Li_Foreground_and_Text-lines_Aware_Document_Image_Rectification_ICCV_2023_paper.pdf). Numbers come from different papers' tables and benchmark variants, so compare them only approximately. D2Dewarp's licence was not checked.

Three things stand out.

**The UVDoc lineage is where the relevant progress is.** AADD, ArbDR and the SyntheticDoc baseline all keep UVDoc's architecture and change the loss, the cascade or the data. AADD adds an axis-aligned loss premised on feature lines being horizontal or vertical in the flat page, which matches a floor plan far better than text-line methods do; on the UVDoc benchmark it reaches AD 0.098 against UVDoc's 0.119 ([AADD](https://arxiv.org/html/2507.15000)). Its grid is still 45x31 and the paper does not discuss folds, so expect smaller global waviness, not sharp creases. ArbDR adds iterative local refinement and is the neural design most likely to shrink residual kinks, yet its authors say folds remain imperfect ([ArbDR](https://arxiv.org/html/2511.23150)). Neither has released code or weights ([AADD repo](https://github.com/chaoyunwang/AADD), [ArbDR repo](https://github.com/chaoyunwang/ArbDR)).

**SyntheticDoc is the only neural result that attacks grid coarseness directly.** It comes from two of the UVDoc authors, trains on a million simulated sheets including explicit fold-and-unfold creases, and reports that moving from a 31x45 grid at 512x720 to a **121x177 grid at 1024x1440** improves DocUNet LD from 5.98 to 5.51 and AD from 0.252 to 0.219 ([SyntheticDoc](https://arxiv.org/html/2609.15503v1)). On a 5000x7000 photo that grid has a node about every 40 px, four times finer than UVDoc. The paper shows no crease-specific evaluation, so the benefit for kinks is an inference. The repository ships generation and training code and the dataset, but the notes could not confirm a released checkpoint ([SyntheticDoc repo](https://github.com/tanguymagne/SyntheticDoc)). Retraining is a serious undertaking: the authors trained 38 epochs on eight RTX 4090s at about 1.5 hours per epoch, on a dataset larger than 5 TB.

**What you can download today is not better than UVDoc.** DocScanner-L, DocRes, D2Dewarp and DewarpNet all score worse than UVDoc on DocUNet geometry, and DvD's small gain costs 151M parameters and multi-step diffusion. Text-line-driven models (DocTLNet, D2Dewarp, TextFlow) get their fine signal from text; D2Dewarp's authors themselves note that text-sparse documents give weak guidance ([D2Dewarp](https://arxiv.org/html/2507.08492)). DocMatcher needs a flat digital template of the document, which a one-off floor plan does not have ([doc-matcher repo](https://github.com/FelixHertlein/doc-matcher)).

## Licences and size, not tract operators, decide what is deployable

tract is not the obstacle. The project pins tract-onnx 0.23.10, and 0.23 registers the operators these architectures need, including GridSample (opset 16 or later), Einsum, LayerNormalization, Gelu, Erf, MatMul, Softmax, Resize, ConvTranspose and Scan ([tract ops](https://raw.githubusercontent.com/sonos/tract/main/onnx/src/ops/mod.rs), [nn ops](https://raw.githubusercontent.com/sonos/tract/main/onnx/src/ops/nn/mod.rs), [grid_sample.rs](https://raw.githubusercontent.com/sonos/tract/v0.23.10/onnx/src/ops/grid_sample.rs)). No registration for ONNX `Loop` was found, so iterative models would have to be exported with the loop unrolled. Nobody loaded any candidate in tract during this research, so compatibility is predicted from source registrations, not tested.

Licences remove most candidates.

- **DocTr, DocTr++, DocGeoNet, DocScanner:** all carry the same custom licence that prohibits commercial use without written permission, requires share-alike under that same licence, and defines the covered "Algorithm" to include data ([DocScanner LICENSE.md](https://raw.githubusercontent.com/fh2019ustc/DocScanner/main/LICENSE.md)). GPL-3.0 does not allow passing a non-commercial restriction on to recipients, so these weights cannot be embedded. A Hugging Face mirror that tags DocScanner-L as MIT contradicts the upstream licence and should not be relied on ([mirror](https://huggingface.co/shihabsikder092/DocScanner-L)).
- **DvD:** AGPL-3.0 ([LICENSE](https://raw.githubusercontent.com/hanquansanren/DvD/master/LICENSE)), and too heavy for CPU in any case.
- **ForCenNet, AADD, ArbDR:** no LICENSE file in the repositories, so all rights reserved by default whatever the README badge says ([ForCenNet repo](https://github.com/caipeng328/ForCenNet), [AADD repo](https://github.com/chaoyunwang/AADD)).
- **DocMatcher and Inv3D-trained models:** MIT code, but trained on Inv3D, which is CC BY-NC-SA 4.0 ([KIT record](https://publikationen.bibliothek.kit.edu/1000161884)). Whether dataset terms bind weights is legally unsettled; a cautious project would not embed them.
- **FDRNet, RDGR, Marior, DocReal, LA-DocFlatten:** no usable weights. DocReal's README says the source cannot be provided "due to commercial restrictions" ([DocReal](https://github.com/irisXcoding/DocReal)).

What survives is short. **UVDoc** is MIT with the model in the repository, and PaddleOCR republishes it under Apache-2.0 with an official 31.7 MB ONNX file ([UVDoc repo](https://github.com/tanguymagne/UVDoc), [PaddlePaddle/UVDoc_onnx](https://huggingface.co/PaddlePaddle/UVDoc_onnx)). **DewarpNet** is MIT code trained on Doc3D, which is now itself published as MIT, and has a ready 189 MB ONNX export measured at about 0.2 s on a laptop CPU under ONNX Runtime ([DewarpNet LICENSE](https://raw.githubusercontent.com/cvlab-stonybrook/DewarpNet/master/LICENSE), [Doc3D LICENSE](https://raw.githubusercontent.com/cvlab-stonybrook/doc3D-dataset/master/LICENSE), [DewarpNet ONNX](https://huggingface.co/mlboydaisuke/DewarpNet-OpenVINO)). It is deployable but scores worse than UVDoc on every benchmark and works from a 256x256 input. **PaperEdge** (two 139.7 MB checkpoints) and **DocRes** (183 MB plus a 713 MB segmentation model as mirrored) are MIT but too large to embed and also score below UVDoc ([PaperEdge repo](https://github.com/cvlab-stonybrook/PaperEdge), [DocRes mirror](https://huggingface.co/presencesw/DocRes)). For MIT repositories whose weights sit on Google Drive with no separate statement, treating the weights as MIT is the common reading, not an explicit grant.

CPU cost is known only for UVDoc: PaddleOCR reports 869.82 ms on a Xeon Gold 6271C in its high-performance mode ([PaddleOCR docs](https://www.paddleocr.ai/latest/en/version3.x/module_usage/text_image_unwarping.html)). No paper reports CPU timings for the others. Because AADD, ArbDR and SyntheticDoc share UVDoc's convolutional architecture, they should drop into the existing pipeline with a grid-size change once weights exist; that is an inference from their papers, not something tested.

## Classical refinement is the most likely fix, and nobody has published it

The classical literature has no finished, evaluated method for "single photo, folded line drawing", and no open-source tool that does it. It does supply the pieces.

**Why the kinks exist.** A planar patch projects by a homography and stays straight, so the kinks come from the paper's height relief seen from a finite distance. A pinhole derivation in the research notes (not taken from any paper) gives an apparent displacement of roughly r·z/H for a point raised by z at horizontal offset r from the spot under a camera at height H. With H = 1 m, a 3 mm crease 300 mm off-centre shifts by about 0.9 mm, around 11 px at this resolution. A horizontal line crossing a vertical crease picks up a slope break proportional to its distance from the centre row, and the break flips sign on the other side of the centre. This is checkable on your photos: if kinks grow with distance from the image centre and reverse across it, the parallax model holds. It also assumes crease heights of 3 to 5 mm and a 1 m shooting distance, which nobody measured.

**Per-panel homographies** are the physically right model for flat panels, but no paper presents one as a finished single-image method. The closest works are restricted or report detection as the hard part: Das et al. 2017 handles only four-fold documents ([poster](https://www3.cs.stonybrook.edu/~sadas/assets/P4/cfposter.pdf)), Luo and Bo 2022 optimise a developable quad mesh and call fold-line detection "still a challenging problem" ([arXiv 2212.08365](https://arxiv.org/abs/2212.08365v1)), and You et al. 2018 need five to ten photos and a structure-from-motion pipeline ([arXiv 1606.00166](https://arxiv.org/abs/1606.00166)). Real unfolded panels also bow, so pure homographies would leave residuals; a grid with more freedom per panel is the safer formulation.

**Frame-based correction** is the simplest building block. Tsoi and Brown (2004) interpolate the interior from four boundary curves with a Coons patch and state that storage folds along one direction are covered by the model ([Tsoi & Brown](https://www.eecs.yorku.ca/~mbrown/pdf/cvpr04_tsoi.PDF)). Using the drawing's printed border frame as those curves would transfer each crease's kink from the frame into the interior. By the parallax derivation this is close to exact for parallel folds of uniform height and only approximate where folds cross. Its results in the paper are visual only.

**Line-straightness grid refinement** is the core of the recommended approach. He et al. (2013) solve a sparse least-squares mesh warp with a line term that penalises only the component of each detected segment perpendicular to its target direction, a shape term, and a boundary term; with angles fixed it is one sparse linear solve of a few hundred unknowns taking milliseconds ([He et al.](https://people.csail.mit.edu/kaiming/publications/iccv13car.pdf)). Kil et al. (2017) add the document-specific prior that line segments are horizontal or vertical in the rectified page, with iterative outlier rejection ([dblp](https://dblp1.uni-trier.de/rec/conf/icdar/KilSKC17.html)). Neither models creases. You et al. show the missing idea in 3D: weight smoothness high along a ridge and low across it so creases stay sharp ([You et al.](https://ar5iv.labs.arxiv.org/html/1606.00166)).

Combining these gives the proposed design, which is the researchers' own synthesis and untested:

1. Trace long near-horizontal and near-vertical lines in the UVDoc output as polylines, including the border frame.
2. Detect creases as full-span, near-axis-aligned lines, using two cues: steps in the paper's shading after ink removal, and the positions where traced lines change slope.
3. Solve for small displacement fields on a grid of 64 to 128 px spacing with extra nodes on the creases. Each traced line is pulled to a constant row or column, frame samples are pinned to the rectangle, and second-difference smoothness is relaxed almost to zero across the creases.
4. Use robust reweighting to drop mis-traced lines, hatching and genuinely oblique content.
5. Compose the result with the UVDoc grid and resample the original photo once.

All of it is linear least squares and basic filtering, which suits pure Rust. `imageproc` provides Hough line detection and Canny edges under MIT, but returns infinite lines, not bent polylines, and no native Rust LSD crate turned up in one search ([imageproc hough](https://docs.rs/imageproc/latest/imageproc/hough/index.html)). Line tracing robustness is most of the work. The method has no information in empty regions of the drawing, and it risks straightening content that is truly not straight.

Existing tools do not help. Zucker's page_dewarp models a single cubic cross-section driven by text spans and is described by its author as a proof of concept ([Zucker](https://mzucker.github.io/2016/08/15/page-dewarping.html)). ScanTailor's dewarping "works best on the low-curved scans", and a fork states its dewarping is "not for fixing photos of wrinkled or skewed paper" ([scantailor-deviant](https://gitea.com/ImageProcessing-ElectronicPublications/scantailor-deviant/releases/tag/v2025.04.02-beta-3)).

**Capture-side remedies** follow from the parallax model: kinks scale with crease height and inversely with camera distance, and vanish under the camera. Pressing the sheet under glass or acrylic, shooting from 2 m or more, and centring the camera should each cut the kinks proportionally. No controlled evidence for the glass sheet's geometric benefit was found; it is inferred. Vendor guidance agrees in spirit: Genius Scan's advice for folded documents is to remove creases before scanning ([Genius Scan blog](https://blog.thegrizzlylabs.com/2022/10/how-to-improve-scan-quality.html)). No documentation was found that any mainstream scanner app corrects fold geometry.

## Ranked shortlist: refine UVDoc's output instead of replacing it

| Rank | Option | Would it fix the kinks? | Effort and blockers |
|---|---|---|---|
| 1 | Crease-aware line and frame grid refinement on top of UVDoc | Most likely of all options where lines are dense; nothing in blank areas. Untested design. | Moderate custom Rust work; no licence, size or runtime issues |
| 2 | Capture changes: flatten under glass, shoot from further, centre the camera | Reduces kinks at the source in proportion to relief and distance; does not help photos already taken | Minutes; watch for reflections |
| 3 | Frame-only Coons or crease-lattice correction (first stage of option 1) | Partly: good for parallel folds, approximate where folds cross | Small; needs frame tracing only |
| 4 | SyntheticDoc 121x177 model | Plausibly smaller kinks (4x finer grid, crease-rich training data); no crease evidence | Weights unconfirmed; retraining needs multi-GPU and more than 5 TB of data |
| 5 | AADD or ArbDR when released | Straighter lines overall; same 45x31 grid, authors concede folds stay imperfect | Wait for code, weights and an actual LICENSE file |
| 6 | DewarpNet | No; worse than UVDoc everywhere | Deployable today, 189 MB |
| Excluded | DocTr family, DocScanner, DvD, ForCenNet, DocRes, PaperEdge, DocMatcher, D2Dewarp | No evidence of better crease handling | Licence, size, missing weights or template requirement |

**What to try next.** First, measure a few kinks on the existing photo against distance from the image centre; that takes minutes and tells you whether the parallax model, and therefore the whole classical approach, applies. Second, re-shoot once with the sheet flattened and from further away to see how much the problem shrinks for free. Third, build option 3 and then option 1 as a refinement stage after UVDoc, keeping UVDoc for the cropping and deskewing it already does well. In parallel, check the SyntheticDoc repository and ETH project page by hand for a checkpoint; if one exists, exporting it is cheap because the architecture is UVDoc's.

## Conclusion

The question assumed the model was the weak link. The evidence says the representation is: a smooth, coarse warp cannot hold a crease, and the field has optimised for page-averaged metrics on A4 text that do not notice one. That is why the strongest lead is not a competing architecture but UVDoc's own lineage with a denser grid, and why a few hundred lines of least-squares code aimed at exactly this content (long axis-aligned lines, a rectangular frame, a handful of full-span creases) has a better chance than any download.

Two uncertainties would change this assessment. If the kinks on your photos do not follow the parallax pattern, the cause is something else, such as UVDoc's own interpolation error near the folds, and a denser neural grid moves up the list. And if SyntheticDoc, AADD or ArbDR publish weights under real licences, each becomes a cheap experiment worth running before more custom code is written.
