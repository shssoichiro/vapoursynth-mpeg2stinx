use super::*;
use failure::bail;
use failure::Error;
use std::convert::TryFrom;
use vapoursynth::core::CoreRef;
use vapoursynth::prelude::*;

#[derive(Debug, Clone, Copy)]
pub enum FilterMode {
    PointBob,
    Spline36Bob,
    Nnedi3,
    Nnedi3CL,
}

impl Default for FilterMode {
    fn default() -> Self {
        FilterMode::Spline36Bob
    }
}

impl TryFrom<i64> for FilterMode {
    type Error = Error;
    fn try_from(mode: i64) -> Result<Self, Self::Error> {
        Ok(match mode {
            0 => FilterMode::PointBob,
            1 => FilterMode::Spline36Bob,
            2 => FilterMode::Nnedi3,
            3 => FilterMode::Nnedi3CL,
            _ => bail!("MPEG2Stinx: mode must be 0, 1, 2, or 3"),
        })
    }
}

impl FilterMode {
    pub fn deint<'core>(
        self,
        core: CoreRef<'core>,
        api: API,
        src: FrameRef<'core>,
    ) -> Result<FrameRef<'core>, Error> {
        match self {
            FilterMode::PointBob => point_bob(core, api, src),
            FilterMode::Spline36Bob => spline36_bob(core, api, src, true),
            FilterMode::Nnedi3 => nnedi3(core, api, src, 3, false),
            FilterMode::Nnedi3CL => nnedi3(core, api, src, 3, true),
        }
    }
}

fn point_bob<'core>(
    core: CoreRef<'core>,
    api: API,
    src: FrameRef<'core>,
) -> Result<FrameRef<'core>, Error> {
    let clip = separate_rows(core, api, src)?;
    point_resize(
        core,
        api,
        clip,
        clip.width(0) as i64,
        2 * clip.height(0) as i64,
    )
}

fn spline36_bob<'core>(
    core: CoreRef<'core>,
    api: API,
    src: FrameRef<'core>,
    process_chroma: bool,
) -> Result<FrameRef<'core>, Error> {
    let clip = separate_rows(core, api, convert(core, api, src, PresetFormat::Gray8)?)?;

    let even = spline36_resize_crop(
        core,
        api,
        select_even(core, api, clip)?,
        clip.width(0) as i64,
        2 * clip.height(0) as i64,
        0.0,
        0.25,
        clip.width(0) as f64,
        clip.height(0) as f64,
    )?;
    let odd = spline36_resize_crop(
        core,
        api,
        select_odd(core, api, clip)?,
        clip.width(0) as i64,
        2 * clip.height(0) as i64,
        0.0,
        -0.25,
        clip.width(0) as f64,
        clip.height(0) as f64,
    )?;
    let clip = interleave(core, api, &[even, odd])?;
    if src.format().id() == FormatID::from(PresetFormat::Gray8) {
        return Ok(clip);
    }

    Ok(if process_chroma {
        shuffle_planes(
            core,
            api,
            &[
                clip,
                u_to_y8(core, api, spline36_bob(core, api, clip, false)?)?,
                v_to_y8(core, api, spline36_bob(core, api, clip, false)?)?,
            ],
            &[0, 0, 0],
            ColorFamily::YUV,
        )?
    } else {
        shuffle_planes(
            core,
            api,
            &[
                clip,
                u_to_y8(core, api, select_every(core, api, clip, 1, &[0, 0])?)?,
                v_to_y8(core, api, select_every(core, api, clip, 1, &[0, 0])?)?,
            ],
            &[0, 0, 0],
            ColorFamily::YUV,
        )?
    })
}
