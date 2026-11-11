# Classical (non-neural) methods for removing fold-crease kinks and residual distortion in photos of folded technical drawings

Reading guide: "Cited Findings" are things read in a source this session (primary paper text where stated, otherwise search-result summaries, flagged as such). "Inferences" are my own reasoning or derivations and have NOT been demonstrated in any paper for this case. Items recalled from memory but not verified this session are in "Gaps" and labelled as unverified.

Bottom line across all questions: no off-the-shelf classical method or open-source tool was found that fixes fold kinks in a single photo of a line drawing. The most promising route is a custom, small, linear least-squares grid refinement driven by traced drawing lines and the border frame, with smoothness relaxed across detected axis-aligned creases (ingredients: Tsoi & Brown boundary interpolation, He et al. line-preserving mesh energy, Kil et al. horizontal/vertical line-segment prior). The cheapest effective remedy is on the capture side (flatten under glass, shoot from farther away, centre the camera).

---

## 1. Piecewise-planar / fold-line models for folded paper (literature)

### Takeaway
The fold-specific literature is thin and mostly either multi-view (You et al., TPAMI 2018) or restricted to a specific fold pattern (Das et al. 2017, four-fold) or dependent on good boundaries (Tsoi & Brown 2004). No paper was found that does "detect fold lines, fit one homography per panel" on a single image as a finished, evaluated method; the closest single-image formulations (Liang et al. planar strips; Luo & Bo 2022 isometric quad mesh) report fold-line detection as the hard, unsolved part.

### Cited Findings
- Tsoi & Brown (CVPR 2004) model the sheet as a ruled surface between two opposite boundary curves, S(u,v) = (1-v)C1(u) + vC2(u), and state explicitly that folds are covered: "in the case of folds, if the fold is along the iso-parametric lines u = u_c, the surface can still be modeled using only the boundary curves" and "the folds made are almost exclusively along iso-parametric lines to allow the material to be folded into equal pieces for storage. Thus, a document with several folds and curl effects can be modeled." (read from the PDF) — [Tsoi & Brown CVPR 2004](https://www.eecs.yorku.ca/~mbrown/pdf/cvpr04_tsoi.PDF)
- Same paper, stated model violation: "folding a page's corner over on itself introduces a fold against the iso-parametric lines". Their model handles folds in ONE direction (all parallel); a 3 x 2 grid of folds (both directions) is not a ruled surface between two boundary curves, though they use the four-curve Coons blend in practice which tolerates some deviation. Their fold demo is an 80 cm x 33 cm art-book fold-out with parallel folds only. — [Tsoi & Brown CVPR 2004](https://www.eecs.yorku.ca/~mbrown/pdf/cvpr04_tsoi.PDF)
- Das, Mishra, Sudharshana, Shilkrot, "The Common Fold" (ACM DocEng 2017, pp. 125-128): single image; uses the document's creases to split the page into four quadrants, samples sparse 2D coordinates per quadrant and fits a polynomial per quadrant (per search summary of the poster). A later paper notes it handles only four-fold documents. (I believe it uses a CNN to locate the creases, i.e. is "lightly learned", but did not verify.) — [poster](https://www3.cs.stonybrook.edu/~sadas/assets/P4/cfposter.pdf), [researchr entry](https://researchr.org/publication/DasMSS17)
- You, Matsushita, Sinha, Bou, Ikeuchi, "Multiview Rectification of Folded Documents" (TPAMI 2018, arXiv 1606.00166): needs "five to ten still images for each document from different viewpoints", handheld. Pipeline: incremental SfM (700-2000 points seen in >= 3 views) -> l1-robust Poisson depth-map reconstruction (IRLS, weights 1/(|r|+eps)) -> ridge detection from the surface Hessian (vertices with |kappa2| > 0.006 are ridge candidates) -> smoothness re-weighted directionally (high along ridge, low across it) so creases stay sharp -> l1 least-squares conformal mapping (LSCM) with ridge-collinearity and boundary-straightness constraints (gamma = 1e3) -> resample. — [ar5iv full text](https://ar5iv.labs.arxiv.org/html/1606.00166), [arXiv abstract](https://arxiv.org/abs/1606.00166)
- You et al. evaluation: 12 sequences, 6 with ground truth; claims higher accuracy than Brown et al. and Zhang et al. on a global and a local (SIFT-flow) distortion metric, but numbers are only in figures; no runtime reported; shading is not corrected (future work); requires enough keypoints for SfM and repetitive textures cause outliers. They dismiss single-image methods (Liang et al., Tian et al.) as relying on strong assumptions about geometry, content and illumination. — [ar5iv full text](https://ar5iv.labs.arxiv.org/html/1606.00166)
- Liang, DeMenthon, Doermann, "Geometric Rectification of Camera-Captured Document Images" (TPAMI 2008): approximates a developable surface with planar strips estimated from texture flow (text), rectifies each strip separately and smooths the seams; the same search summary mentions a separate content-independent model-based method for documents folded in thirds (search-result summary, not read) — [ResearchGate entry](https://www.researchgate.net/publication/5575035_Geometric_Rectification_of_Camera-Captured_Document_Images)
- Luo & Bo, "Geometric Rectification of Creased Document Images based on Isometric Mapping" (arXiv 2212.08365, 2022): non-neural optimisation framework; quad mesh as a discrete developable surface + pinhole camera; priors are developability and straight feature lines in the image; works without feature lines but "reliably detected feature lines improve the result considerably"; authors say feature/fold-line detection "is still a challenging problem". No code link found, no quantitative numbers in abstract. — [arXiv 2212.08365](https://arxiv.org/abs/2212.08365v1)
- Brown & Seales-style 3D approaches: Tsoi & Brown characterise them as needing an acquired 3D reconstruction and a slow spring-relaxation flattening, with "additional 3D shape information rarely available". — [Tsoi & Brown CVPR 2004](https://www.eecs.yorku.ca/~mbrown/pdf/cvpr04_tsoi.PDF)
- General limitation of smooth parametric models for folds: "it is not always practical to model baselines (especially in the presence of folds) with a smooth curve"; cylinder/polynomial models "can only model relatively smooth variations". — [arXiv 2003.06872](https://arxiv.org/pdf/2003.06872)
- A patent on folded IDs notes the fold has low contrast and is hard to detect, and searches for it in a narrow corridor predicted from the document's known aspect ratio (search summary). — [US 10699146](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/10699146)

### Inferences
- A per-panel homography model is physically right for the photo (see section 5), but the input here is the UVDoc output, i.e. the photo after a smooth non-projective warp. Each panel is then "homography composed with a smooth warp", so pure per-panel homographies fitted on the UVDoc output will leave residuals. Either fit on the original photo (using the UVDoc grid only to locate page and panels) or use a model with more freedom per panel (homography + low-order smooth residual, or a grid as in section 2).
- For a 3x2..3x3 panel layout, panel homographies must agree along shared creases (C0 continuity). A convenient parametrisation is a control lattice with nodes at border/crease intersections (4x3 or 4x4 nodes) and a projective or bilinear map per panel; continuity is then automatic. Bilinear per panel is not exactly projective, but the perspective variation across a 20 cm panel tilted by 1-2 degrees is negligible.
- The UVDoc 45x31 grid over ~5000x7000 px has ~160 px node spacing, so it cannot represent a slope discontinuity sharper than one cell; residual kinks at creases are an expected consequence of the smooth coarse grid rather than a failure that a better smooth model would fix.

### Gaps
- Could not access full text of Das et al. 2017 (how creases are located, accuracy numbers).
- Tian & Narasimhan (CVPR 2011), Meng et al. (developable surfaces; a CVPR 2014 paper "Active Flattening of Curved Document Images via Two Structured Beams" surfaced by title only), Zhang & Tan folded-document papers, Brown & Seales originals: not read this session. From memory (unverified): Tian & Narasimhan need dense text for line/stroke tracing, so do not fit a line drawing.
- "Axis-Aligned Document Dewarping" (arXiv 2507.15000, 2025) surfaced in search; it appears to be neural with an axis-alignment prior; not read (other researchers cover neural).
- No paper found that evaluates any method specifically on technical drawings / plans.

---

## 2. Line-straightness based refinement (mesh / grid optimisation)

### Takeaway
The standard formulation is a sparse least-squares mesh warp with a line term, a shape/smoothness term and a boundary term (He et al. ICCV 2013), and document-specific variants add "line segments should be horizontal or vertical" (Kil et al. ICDAR 2017). None of them models creases; smooth-surface models (Zucker's cubic sheet, cylinders, polynomials) structurally cannot represent a kink. A crease-aware variant is straightforward to formulate but is my own proposal, not a published result.

### Cited Findings
- He, Chang, Sun, "Content-Aware Rotation" (ICCV 2013), read from the PDF: regular quad mesh (~400 quads); lines extracted with LSD (IPOL code) and cut at mesh edges so each piece lies inside one quad; each endpoint is a bilinear combination of the 4 quad vertices, so a segment direction e_k = P_k V is linear in vertex vector V. — [He et al. ICCV 2013](https://people.csail.mit.edu/kaiming/publications/iccv13car.pdf)
- Their line energy: E_L = (1/K) sum_k ||(R_k U_k R_k^T - I) e_k||^2 with U_k = u_k(u_k^T u_k)^-1 u_k^T (u_k = input direction, R_k = rotation by target angle). It is quadratic in V and penalises only the component of the deformed segment perpendicular to the target direction (scale is free). — [He et al. ICCV 2013](https://people.csail.mit.edu/kaiming/publications/iccv13car.pdf)
- Other terms: shape E_S = as-similar-as-possible per quad, (1/N) sum_q ||(A_q(A_q^T A_q)^-1 A_q^T - I) V_q||^2; boundary E_B pins boundary vertices to the output rectangle (x=0, x=w, y=0, y=h) with weight 1e8; lambda_L = lambda_R = 100. Lines in "canonical" (horizontal/vertical) bins get a hard target angle; others are relaxed. — [He et al. ICCV 2013](https://people.csail.mit.edu/kaiming/publications/iccv13car.pdf)
- Cost: with theta fixed the problem is one sparse linear solve in a few hundred unknowns ("a few milliseconds"); full optimisation 0.1-0.2 s; resampling an 8 MP image ~0.5 s (C++, 2013 desktop). — [He et al. ICCV 2013](https://people.csail.mit.edu/kaiming/publications/iccv13car.pdf)
- A Python/TensorFlow reproduction exists (Windows-only tested; licence not checked). — [ForeverPs/content-aware-rotation](https://github.imc.re/ForeverPs/content-aware-rotation)
- Kil, Seo, Koo, Cho, "Robust Document Image Dewarping Method Using Text-Lines and Line Segments": published at ICDAR 2017 (pp. 865-870, DOI 10.1109/ICDAR.2017.146), NOT CVPR. Adds to the text-line cost a term encoding that "many of the line segments in the image are horizontally or vertically aligned in the well-rectified images", with iterative rejection of badly aligned text-lines/segments because many segments are outliers. (search summaries; full text not read) — [dblp](https://dblp1.uni-trier.de/rec/conf/icdar/KilSKC17.html), [summary](https://www.bohrium.com/paper-details/robust-document-image-dewarping-method-using-text-lines-and-line-segments/813062709439365120-102493)
- Kim, Koo, Cho, "Document dewarping via text-line based optimization" (Pattern Recognition 48(11), 2015): cost based on straightness / vertical position of text lines in the rectified plane plus line-spacing and alignment terms, optimised with Levenberg-Marquardt. (summary only) — [summary](https://www.bohrium.com/paper-details/document-dewarping-via-text-line-based-optimization/814630705257512961-2432)
- Zucker's page_dewarp: page surface = a curve swept along y; "I model the horizontal cross-section of the page surface as a cubic spline whose endpoints are fixed at zero", fully determined by two end slopes alpha, beta; plus pose r, t; objective = reprojection error of keypoints on detected text spans; solved with scipy Powell (100-600 parameters, 4-23 s); only vertical distortion modelled; described as "mostly intended as a proof-of-concept". Horizontal rules are used only as a fallback when text is scarce. — [Zucker blog](https://mzucker.github.io/2016/08/15/page-dewarping.html)

### Inferences
- Why page_dewarp does not fit: (a) a single cubic cross-section with 2 shape parameters is C2-smooth and cannot produce slope discontinuities at 2 vertical creases, let alone creases in both directions; (b) it is driven by text spans, which a floor plan lacks; (c) it assumes a generalized cylinder (curvature along one axis only).
- Proposed crease-aware grid refinement on the UVDoc output (all linear least squares, pure-Rust friendly). Because the residual is small (few px to a few tens of px) and lines are already near axis-aligned, use displacement fields rather than a full 2D mesh:
  - Unknowns: dy(x,y) and dx(x,y) on a regular grid (e.g. 64-128 px spacing, with extra grid columns/rows placed exactly on detected creases), plus one unknown offset c_k per traced line.
  - Line term (horizontal lines): for samples (x_i, y_i) on traced near-horizontal line k: (y_i + B(x_i,y_i)·dy - c_k)^2, with B the bilinear interpolation weights. Vertical lines analogously with dx. This is He et al.'s "perpendicular component only" energy specialised to axis-aligned targets, with c_k eliminating the unknown line position. Long lines spanning several panels are what carry information across creases.
  - Border term: samples on the detected frame sides constrained to the frame rectangle (known c, high weight) — fixes the gauge and global scale/shear.
  - Smoothness: second differences of dy, dx in both directions (thin-plate-like), with the weight of the second difference ACROSS a crease reduced to ~0-5 % at crease nodes (direct analogue of You et al.'s ridge-aware directional re-weighting, here in 2D). This lets the field have a slope break exactly at the crease while staying smooth inside panels.
  - Small Tikhonov term on dy, dx to stay near UVDoc where there is no evidence.
  - Robustness: IRLS (Huber/l1 weights, as in You et al. and Kil et al.'s iterative rejection) to drop mis-traced lines, dimension lines, hatching, stairs/arcs.
  - Solve: sparse normal equations with conjugate gradient; unknown count ~2 x (60 x 90) + lines, trivial. Apply by composing with the UVDoc map and resampling the original photo once (avoid double interpolation).
- Note the cross-coupling the 1D split ignores: dy mostly straightens horizontal lines and dx vertical ones, which matches the physics (section 5: a raised fold displaces points radially; a line parallel to the crease stays straight, a line crossing it gets kinked). Orthogonality is not enforced by the split but the frame constraint handles global shear.
- Line input need not be LSD. On a UVDoc-rectified drawing, tracing is simpler: 1D derivative-of-Gaussian / dark-ridge response perpendicular to the expected direction, per-column minima linked by dynamic programming or greedy tracking within +-N px, keep tracks longer than ~10 % of page width. Polyline tracks (not straight segments) are what is needed, since the lines are bent.
- Expected benefit: good in regions dense with long wall/grid/frame lines; no information in empty regions (the smoothness prior interpolates). Risk: genuinely non-straight or slightly oblique content pulled straight — gate on |angle| < ~1.5 degrees and robust weights. Difficulty: moderate (line tracing robustness is most of the work; the solver is easy).
- Thin-plate splines are a poor fit for kinks (globally smooth, dense solve); a grid with locally reduced smoothness is preferable.

### Gaps
- Full text of Kil et al. 2017 and Kim et al. 2015 (exact cost terms, weights, runtimes) not accessible; no public code found for either.
- Carroll et al. (line-preserving wide-angle/perspective warps) not read; He et al. cite that line of work as the origin of the "angle coupling" line energy.
- Plumb-line lens calibration literature not researched; inference only: it fits a low-order global radial model and cannot express localised kinks, but should be considered separately if the phone's residual lens distortion bends long lines globally.
- No published evidence that a crease-aware line-driven grid refinement works on folded plans; the formulation above is untested.

---

## 3. Using the drawing's rectangular border frame (boundary-based dewarping)

### Takeaway
Tsoi & Brown's boundary interpolation is directly applicable in principle and trivially implementable (four boundary curves + bilinearly blended Coons patch), and it explicitly claims to handle storage folds; but it only transfers what the boundary shows into the interior, so it fixes kinks of interior lines only insofar as each crease's effect is visible on the frame and varies linearly across the sheet. It is a good initialisation/constraint, not a full solution.

### Cited Findings
- Method (read from PDF): four boundary curves c1..c4 (top, right, bottom, left) as natural cubic splines through extracted boundary points; mapping from rectified (u,v) to image (x,y) by the bilinearly blended Coons patch c(u,v) = (1-u)c4(v) + u c2(v) + (1-v)c1(u) + v c3(u) - [bilinear blend of the four corners]; output by bilinear resampling. Assumes "the printed content will be rectilinear in its true planar format". — [Tsoi & Brown CVPR 2004](https://www.eecs.yorku.ca/~mbrown/pdf/cvpr04_tsoi.PDF)
- Parameterisation matters: arc-length parameterisation of the 2D boundary curves gives "acceptable results ... when the depth distortion is small compared to the size of the imaged material"; with significant depth change they needed a "uniform" parameterisation obtained from a checkerboard strip placed under the material, otherwise the result shows uneven spacing along the boundary. — [Tsoi & Brown CVPR 2004](https://www.eecs.yorku.ca/~mbrown/pdf/cvpr04_tsoi.PDF)
- Small non-linearities such as radial lens distortion are absorbed by the boundary splines. — [Tsoi & Brown CVPR 2004](https://www.eecs.yorku.ca/~mbrown/pdf/cvpr04_tsoi.PDF)
- Shading: illumination image L(u,v) estimated by the same Coons interpolation of luminance sampled along the blank white margin just inside the boundary; reflectance R = exp(log I - log L). Their estimated illumination images show the fold shading as vertical bands. — [Tsoi & Brown CVPR 2004](https://www.eecs.yorku.ca/~mbrown/pdf/cvpr04_tsoi.PDF)
- Boundary acquisition in the paper: checkerboard corners for experiments; "under controlled imaging environment, automatic border detection can be robustly performed using segmentation". A later CVPR 2007 multi-view paper by the same group notes the single-view method needed a 3D pattern and white borders (search snippet). — [Tsoi & Brown CVPR 2004](https://www.eecs.yorku.ca/~mbrown/pdf/cvpr04_tsoi.PDF), [Brown et al. CVPR 2007](https://www.eecs.yorku.ca/~mbrown/pdf/cvpr07_multiview.pdf)
- You et al. also use boundary-straightness constraints in their flattening step. — [ar5iv](https://ar5iv.labs.arxiv.org/html/1606.00166)

### Inferences
- For this case, use the printed border frame (a long dark line, easier and more accurate to trace than the paper edge on a wooden floor) as c1..c4. The sheet is nearly flat, so the small-depth condition for arc-length parameterisation holds. Better: because creases cross the frame, split each side's parameterisation at crease crossings and at the corners (piecewise arc-length), which is exactly a "control lattice at crease/frame intersections".
- What a Coons patch on the frame fixes: a kink on the top frame line at a vertical crease is transferred to interior horizontal lines with a weight that falls off linearly to the bottom line's own kink. From section 5, the true kink magnitude of a horizontal line at a vertical crease is proportional to its y-distance from the camera's nadir point, i.e. it IS linear in y when the crease height is constant along its length. So for folds in one direction and uniform crease height, linear blending between top and bottom frame kinks is close to exact. With crossing folds (heights vary along a crease, pyramidal bumps at intersections), it is only approximate.
- Recommended role: boundary conditions and gauge for the grid refinement in section 2 (frame samples constrained to the rectangle), with interior lines supplying what the boundary cannot.
- Tsoi & Brown's margin-based illumination interpolation is too weak for crease shading in the interior of a 2D fold grid; a direct background estimate is better (section 4).

### Gaps
- No quantitative accuracy for Tsoi & Brown (results are visual only).
- PaperEdge and other boundary-driven neural methods were out of scope and not researched.
- Did not find any work using an inner printed frame (as opposed to the paper edge) as the boundary.

---

## 4. Crease detection

### Takeaway
There is little dedicated literature on single-image crease detection; published systems either detect ridges in reconstructed 3D (You et al.), use a learned detector (Das et al., unverified), or admit the problem is open (Luo & Bo). For this case the strong priors (axis-aligned, full-span, few lines, on a nearly binary drawing) make a simple 1D projection approach plausible, but that is inference.

### Cited Findings
- You et al. detect ridges geometrically from the Hessian of the reconstructed depth map (second principal curvature threshold), not from the image. — [ar5iv](https://ar5iv.labs.arxiv.org/html/1606.00166)
- Luo & Bo: detection of feature/fold lines "is still a challenging problem"; another summary of the paper says fold-line detection is hard especially where folding angles are large. — [arXiv 2212.08365](https://arxiv.org/abs/2212.08365v1)
- Folded-ID patent: fold has low contrast; search restricted to a narrow corridor predicted from known document proportions. — [US 10699146](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/10699146)
- Tsoi & Brown's estimated illumination images show folds as band edges in luminance, i.e. shading is a usable cue for fold location. — [Tsoi & Brown CVPR 2004](https://www.eecs.yorku.ca/~mbrown/pdf/cvpr04_tsoi.PDF)

### Inferences
- Shading cue (pure Rust, basic ops): (1) downscale to ~1000 px; (2) estimate paper background by removing ink: grey-scale morphological closing or a large-window high percentile/median (window > max line width), then light blur; (3) take log; (4) for vertical creases compute the horizontal derivative and aggregate along each column with a robust statistic (median or trimmed mean of the signed derivative and of its absolute value); (5) peaks of the column profile, non-max suppressed, are crease candidates; same for rows. A mountain/valley fold appears as a step in shading (the two panels have different normals) and often a thin bright/dark ridge at the crease itself, so use both the first-derivative profile (step) and a second-derivative / ridge profile. Full-span aggregation is what gives robustness; this is a degenerate Hough transform restricted to near-0 and near-90 degrees (allow +-1 degree by testing a few shears or by tracking per horizontal band).
- Geometric cue: kink locations of traced lines. For each traced near-horizontal polyline compute a robust slope before/after each x (two-window fit); accumulate |slope change| weighted by line length into a histogram over x. Peaks that coincide with shading peaks are creases. This also works when lighting is flat, and directly measures what must be corrected.
- Priors: creases span the full sheet, are near axis-aligned after UVDoc, and filing folds give panels of near-equal or standard width; candidate positions can be scored against a regular-spacing model (as in the patent's corridor search). Number of creases is small (2 vertical, 1-2 horizontal expected here).
- Drawing lines themselves are dark and axis-aligned and will dominate naive gradient projections; ink removal (step 2) is essential, and long printed lines should be masked using the line tracer's output.
- Frangi-style multi-scale Hessian ridge filters are overkill here; a 1D second-derivative-of-Gaussian across the crease direction after ink removal is the axis-aligned special case.
- Wood-floor background is irrelevant after cropping, but uneven room lighting produces low-frequency gradients: high-pass the background (subtract a very large blur) before projecting.

### Gaps
- No source found that benchmarks single-image fold-line detectors; robustness under diffuse overhead lighting (weak crease shading) is unknown and must be tested on the real photos.
- Standard fold schemes for drawings (from memory, unverified: DIN 824 folds large sheets to A4 with 210 mm / 190 mm panel widths and 297 mm heights) would give a strong positional prior; not verified this session.

---

## 5. Physical magnitude of crease kinks; is it parallax?

### Takeaway
No literature quantifying this specific situation was found. A pinhole derivation (my own) says the kinks are a pure parallax effect of the paper's height relief: a few mm of crease height seen from ~1 m gives displacements up to roughly 1 mm on paper (~10 px at this resolution) and slope breaks of up to ~0.5-1 degree, growing with distance from the point directly below the camera and shrinking proportionally with camera distance. For exactly planar panels a per-panel homography models this exactly.

### Cited Findings
- Projection of a planar patch is a homography, and lines are preserved under projection; Tsoi & Brown rely on exactly this for their ruled-surface argument ("since lines are preserved under projection ..."). — [Tsoi & Brown CVPR 2004](https://www.eecs.yorku.ca/~mbrown/pdf/cvpr04_tsoi.PDF)
- Depth relief matters in practice: Tsoi & Brown observe visible residual artefacts ("uneven black and white square pattern along the boundaries") when depth variation is ignored for a strongly non-flat fold-out, and acceptable results when "depth distortion is small compared to the size of the imaged material". — [Tsoi & Brown CVPR 2004](https://www.eecs.yorku.ca/~mbrown/pdf/cvpr04_tsoi.PDF)

### Inferences (derivation, not from literature)
- Setup: camera centre at height H above the floor, looking straight down; nadir point N on the sheet. A paper point at horizontal offset r from N raised by height z is imaged where a floor-level point at offset r·H/(H-z) ≈ r(1 + z/H) would be. Apparent displacement, in sheet units: d ≈ r·z/H, directed radially away from N.
- Numbers: A2 = 420 x 594 mm; ~7000 px along 594 mm ≈ 11.8 px/mm. With H = 1000 mm, z = 3 mm, r = 300 mm: d ≈ 0.9 mm ≈ 11 px. With z = 5 mm at the corner (r ≈ 360 mm): ≈ 1.8 mm ≈ 21 px. At N itself: zero.
- Kink of a line crossing a crease: tent profile z(x) = h - s·|x - x0| near a vertical crease at x0 (s = panel slope, e.g. 3 mm rise over 100 mm => s = 0.03 ≈ 1.7 degrees). A horizontal line at vertical offset Y from N gets displaced in y by Y·z(x)/H, so its slope changes across the crease by Δ ≈ 2·Y·s/H. Y = 200 mm, H = 1000 mm: Δ ≈ 0.012 rad ≈ 0.7 degrees, i.e. ~0.6 mm (~7 px) of deviation per 100 mm of line. For Y = 0 (line passing under the camera) no kink at all. Lines parallel to the crease are shifted but stay straight. The kink flips sign on opposite sides of N.
- So: (a) kinks are camera-position dependent parallax, not paper stretching; (b) magnitude scales as 1/H — shooting from 2 m halves it, 3 m thirds it; (c) centring the camera over the sheet minimises the worst case; (d) pattern check for the real images: kinks of horizontal lines at a vertical crease should grow linearly with distance from the image centre row and reverse sign across it. If observed, this confirms the model and allows a very low-parameter fit (one height profile per crease).
- Foreshortening from panel tilt is negligible: cos(1.7 degrees) ≈ 0.99956, i.e. 0.09 mm over a 200 mm panel (~1 px).
- Per-panel homography is exact only for flat panels. Real unfolded paper is stiff: panels bow, and the crease is a tight but finite-radius bend, with dome/saddle shapes at crease intersections. Expect per-panel homographies to remove most but not all of the kink; a low-order residual per panel (or the grid model of section 2) covers the rest.
- A minimal physical model alternative to a free grid: height field z(x,y) on the sheet (piecewise-linear across creases, smooth inside panels) + known/estimated nadir point and H; displacement = (p - N)·z/H. One scalar field instead of two displacement fields, and it couples horizontal and vertical line evidence. Requires approximate camera geometry (nadir from EXIF-less heuristics = image centre of the original photo if shot straight down). More constrained, slightly harder; untested.

### Gaps
- No paper found that measures crease height of unfolded drawings or resulting image-space error; the numbers above assume h = 3-5 mm and H = 1 m and should be checked against the actual photos (measure kink angle vs. distance from image centre).
- Phone lens distortion residuals (after the phone's internal correction) were not quantified; could contribute smooth, global bending but not kinks.

---

## 6. Capture-side remedies

### Takeaway
I found no authoritative documentation that any mainstream phone scanner app geometrically corrects fold creases; vendor guidance is to flatten the paper before scanning. Geometry (section 5) says the effective capture-side fixes are: reduce relief (glass/acrylic sheet, weights, reverse-folding), increase camera distance, centre the camera; multi-view reconstruction works in the literature but needs 5-10 photos and an SfM pipeline.

### Cited Findings
- Genius Scan's own blog: when a document has been folded, getting a flat scan gets harder; advice is to remove creases as much as possible before scanning (per search summary of the vendor post). — [Genius Scan blog](https://blog.thegrizzlylabs.com/2022/10/how-to-improve-scan-quality.html)
- Sheet-fed scanner vendors give the same advice (smooth out creases before feeding). — [Ricoh/PFU manual](http://origin.pfultd.com/downloads/IMAGE/manual/fi-7300nx/OG/en/C_OG/C_OG/topic/scan_half.html)
- Forum report (anecdotal): on a flatbed, different exposures and weighing the paper down with plexiglass did not remove fold marks; ironing at low heat was suggested. Note this concerns the visible crease mark (shading/paper damage), not geometry. — [comp.periphs.scanners thread](https://comp.periphs.scanners.narkive.com/GLdAvy1K/remove-creases-folds-when-scanning-documents)
- Samsung One UI 8.5 scanner has a "corner fold" removal feature (dog-ears, generative), not full-sheet fold rectification. Online "remove crease" services claim to handle bi-/tri-fold shadows; these are unverified vendor claims and concern shading. — [Android Authority](https://www.androidauthority.com/samsung-galaxy-phones-document-scanner-one-ui-8-5-hands-on-3730608/), [phototoscan](https://www.phototoscan.com/features/remove-crease/)
- Multi-view works for folds: You et al. rectify folded letters/receipts from 5-10 handheld photos, preserving creases via ridge-aware reconstruction. No code release found. — [ar5iv](https://ar5iv.labs.arxiv.org/html/1606.00166)
- A physical calibration strip helps: Tsoi & Brown placed a checkerboard under/around the material to recover correct parameterisation under depth variation, arguing props are acceptable in archival digitisation. — [Tsoi & Brown CVPR 2004](https://www.eecs.yorku.ca/~mbrown/pdf/cvpr04_tsoi.PDF)
- Active/structured-light flattening exists in the literature (Meng et al., CVPR 2014, two structured laser beams) — title only. — [Meng et al. CVPR 2014](https://openaccess.thecvf.com/content_cvpr_2014/papers/Meng_Active_Flattening_of_2014_CVPR_paper.pdf)

### Inferences
- Ranked by benefit/effort for this user: (1) flatten: glass or acrylic sheet on top (watch reflections: light from the sides at a shallow angle, no flash, dark ceiling), or weights/tape at corners and reverse-creasing the folds; reducing h from 4 mm to 1 mm cuts kinks 4x. (2) Distance: shoot from 2-2.5 m with the tele lens (or crop the main camera); kinks scale as 1/H, at the cost of resolution if cropping. (3) Centre the camera over the sheet and keep it parallel. (4) Diffuse, even lighting from two sides to weaken crease shading (helps whitening, though it removes the shading cue for crease detection — the geometric kink cue remains).
- Two shots from different positions give different parallax for the same relief; in principle the height field could be solved from the disparity between the two rectified images (a 2-view special case of You et al.), but this is a research project, not a quick win.
- Phone LiDAR / ToF depth has roughly cm-level accuracy and low resolution, so few-mm crease relief is likely below its noise floor (unverified).

### Gaps
- No official statements found for Google PhotoScan, Adobe Scan, Microsoft Lens or Genius Scan on handling folds geometrically; from memory (unverified) PhotoScan's multi-shot capture targets glare removal on photo prints, not geometry of folds.
- No evidence found (either way) on LiDAR-based document flattening accuracy.
- No controlled evidence on glass-platen benefit for geometry specifically; the benefit is inferred from the parallax model.

---

## 7. Existing open-source implementations and Rust building blocks

### Takeaway
Nothing reusable solves the problem directly. Existing dewarpers (page_dewarp, ScanTailor, Leptonica) assume smooth book-like curvature and/or text lines. For a pure-Rust implementation the available building blocks are imageproc (Hough, edges, MIT) and a hand-written sparse solver; no Rust-native LSD/EDLines crate was found.

### Cited Findings
- page_dewarp (Matt Zucker): MIT licence; Python with scipy, OpenCV >= 3.0, PIL; cubic-sheet + text-span model as in section 2; README mentions no ports. — [GitHub](https://github.com/mzucker/page_dewarp), [blog](https://mzucker.github.io/2016/08/15/page-dewarping.html)
- Zucker notes Leptonica and the CTM method also handle horizontal distortion, which his does not. — [blog](https://mzucker.github.io/2016/08/15/page-dewarping.html)
- ScanTailor Advanced: GPL-3.0(+). Its dewarping builds a "distortion model" from a mesh whose control points lie along the top and bottom content borders (curves) and dewarps accordingly; project text says it "works best on the low-curved scans"; auto mode uses an empirical 2.75-degree vertical-boundary threshold. When Scan Tailor builds the grid itself the curves are polylines (secondary source). — [gitea mirror](https://gitea.com/ImageProcessing-ElectronicPublications/scantailor-advanced), [Guix package](https://packages.guix.gnu.org/packages/scantailor-advanced), [spline fitting post](https://hn.jakechampion.name/reader/https://tulon.github.io/spline-fitting)
- scantailor-deviant release note describes its new dewarping as for sheets curved along a cylindrical surface only, "not for fixing photos of wrinkled or skewed paper". — [scantailor-deviant release](https://gitea.com/ImageProcessing-ElectronicPublications/scantailor-deviant/releases/tag/v2025.04.02-beta-3)
- imageproc 0.27.0 (MIT): `hough` module with `detect_lines` (binary image -> `PolarLine` list), `LineDetectionOptions`, `intersection_points`, drawing helpers; also has edges (Canny). It returns infinite lines in polar form, not segments or polylines. — [docs.rs imageproc::hough](https://docs.rs/imageproc/latest/imageproc/hough/index.html)
- LSD/EDLines in Rust: search found only the `opencv` crate bindings (line_descriptor module) and C/C++/Python implementations; no native crate. LSD reference implementation is the IPOL one used by He et al. — [opencv crate docs](https://docs.rs/crate/opencv/0.84.5/source/docs/line_descriptor.rs), [IPOL LSD](http://www.ipol.im/pub/art/2012/gjmr-lsd/)
- Content-aware rotation reproduction (Python LSD + TensorFlow mesh warp). — [repo](https://github.imc.re/ForeverPs/content-aware-rotation)
- No code found for You et al., Luo & Bo, Kil et al., Kim et al., Tsoi & Brown. — [arXiv 1606.00166](https://arxiv.org/abs/1606.00166), [arXiv 2212.08365](https://arxiv.org/abs/2212.08365v1)

### Inferences
- ScanTailor's model (two bounding curves top/bottom + straight vertical generators) is essentially the Tsoi & Brown ruled-surface model; with polyline curves it could represent kinks at vertical creases but not horizontal creases, and it is interactive/GPL C++ — useful as a design reference only.
- Hough (imageproc) is suited to finding the frame sides and crease candidates as global near-axis-aligned lines, but not to measuring bends; the refinement needs polyline tracing, which is simple to hand-write for near-axis-aligned lines (section 2).
- Everything required is basic: separable Gaussian/derivative filters, grey morphology or percentile filter, 1D projections, DP line tracking, sparse CG solve (hand-rolled, or a small pure-Rust linear algebra crate), bilinear/bicubic remap (the tool already resamples with a grid).
- Suggested implementation order by value/effort: (1) frame detection + Coons/lattice correction from frame polylines; (2) crease detection (shading + kink voting); (3) crease-aware line-driven grid refinement; (4) optional height-field model.

### Gaps
- Leptonica dewarp and unpaper were not checked this session. From memory (unverified): Leptonica's dewarp builds vertical then horizontal disparity fields from text lines (needs substantial text; BSD-2-Clause); unpaper does deskew/border cleanup only, no dewarping (GPL-2). "libdewarp": no project by that name was found.
- Did not verify licences of the IPOL LSD code (from memory AGPL) or EDLines implementations; did not search crates.io directly for small pure-Rust LSD ports, sparse solvers or mesh-warp crates, so absence is "not found in one web search", not confirmed absence.
- Rust/C++ ports of page_dewarp exist according to general knowledge but none was confirmed this session.
