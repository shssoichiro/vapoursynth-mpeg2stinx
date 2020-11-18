# Vapoursynth-Mpeg2Stinx

## Installation

#### Arch Linux

Probably there will be an AUR package for it soon. Until then, have Rust installed and then:

```bash
cargo build --release
sudo cp target/release/libvapoursynth_mpeg2stinx.so /usr/lib/vapoursynth
```

#### Other Operating Systems

Same as above but the vapoursynth plugin directory is probably different.

### Dependencies

- For `mode = 2`: [nnedi3](https://github.com/dubhater/vapoursynth-nnedi3)
- For `mode = 3`: [nnedi3cl](https://github.com/HomeOfVapourSynthEvolution/VapourSynth-NNEDI3CL)
- For `order != -1`: [yadifmod](https://github.com/HomeOfVapourSynthEvolution/VapourSynth-Yadifmod)

## Usage

#### Basic usage

```python
import vapoursynth as vs
core = vs.get_core()

# This is an example, obviously your source will be different
clip = core.ffms2.Source(source='example.mkv')

clip = core.mpeg2stinx.Mpeg2Stinx(clip)

clip.set_output()
```

#### Optional arguments

##### `mode`: int

Default: `1`

Resizer used for interpolating fields to full size.
- 0: PointResize
- 1: Spline36Resize
- 2: nnedi3
- 3: nnedi3cl

Mode 1 kills vertical detail a bit less aggressively than mode 0,
at the cost of some speed.

Mode 2 is slower yet but produces pretty much identical results
to mode 1, so don't use it unless your encode is running too fast.

Mode 3 is identical to mode 2, but runs on the GPU, so it should be faster
if you have a capable GPU. But probably still slower than mode 1.

##### `sw`: int, `sh`: int

Default: `1`, `1`

Parameters for the size of the rectangle on which to perform
min/max clipping, `(2sw+1)×(2sh+1)`.

Using small values (e.g. `sw=sh=1`) generally results in
faster processing, but also tends to transfer artifacts
from the dirty field to the clean field in flat areas.

Using large values (e.g. `sw=sh=3`) preserves more detail
and cleans flat areas better, but also keeps more artifacts
around edges and munges interlaced areas (e.g. scrolling text overlays)
more badly.

Both sw and sh must be non-negative and do not have to be equal.

##### `contra`: bool

Default: `1`

Whether to use contrasharpening.

#### `blurv`: float

Default: `0.9` if `contra`, else `0.0`

How much vertical blur to apply.

Positive values call Avisynth's own Blur
and non-positive values don't apply any blur
(but don't sharpen either).

Using a value close to 1.0 together with `contra=1`
reduces combing à la Vinverse.

##### `sstr`: float

Default: `2.0`

Contrasharpening strength.

##### `scl`: float

Default: `0.25`

Contrasharpening scale.

##### `dither`: bool

Default: `0`

Whether to dither when averaging two clips.

`mt_average` has a slight rounding bias which can be avoided
by working at 16-bit precision, but if you have to use this filter at all,
the source is probably bad enough that this small bias is irrelevant.
Additionally, as the contrasharpening is done with 8-bit intermediates,
dithering is unlikely to be useful if contrasharpening is also enabled.

##### `order`: int

Default: `-1`

Field order to use for `yadifmod`. This field order should be the opposite
of the source (pre-IVTC) field order, as field matching for the combed frames
effectively inverts the field order. This helps to limit damage done
in static scenes or to static text overlays (e.g. non-scrolling credits)
and has little effect on artifact removal.

- -1: `yadifmod` unused.
- 0: Motion mask using `yadifmod`, bottom-field-first.
- 1: Motion mask using `yadifmod`, top-field-first.

##### `diffscl`: float

Default: `None`

If specified, temporal limiting is used, where the changes by `crossfieldrepair`
are limited to `diffscl` times the difference between the current frame
and its neighbours.
This is independent of the `yadifmod` limiting above; using both together
is allowed but generally a waste of CPU cycles.

`diffscl` must be non-negative or undefined, where a sane value would be about 2.
Larger values result in less limiting (i.e. more damage to static areas),
while a value of 0 results in this filter becoming a slower version of `Vinverse`
(or a really expensive no-op if contrasharpening is also disabled).

Additionally, unlike `yadifmod`'s temporal limiting, this does affect artifact removal.
This tends to attenuate severe discoloration around scene changes caused by
particularly awful MPEG-2 encoders, which has previously resisted attempts at automatic removal.
It is, however, expected that temporal limiting (either with this option or yadifmod)
will be considerably less useful on live action or CG sources than on animated ones.
