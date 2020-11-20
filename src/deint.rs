use super::*;
use failure::bail;
use failure::Error;
use std::convert::TryFrom;
use vapoursynth::core::CoreRef;
use vapoursynth::prelude::*;
use vapoursynth::video_info::Property::Constant;

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
            _ => bail!("Mpeg2Stinx: mode must be 0, 1, 2, or 3"),
        })
    }
}

impl FilterMode {
    pub(crate) fn deint<'core>(
        self,
        core: CoreRef<'core>,
        api: API,
        src: &Node<'core>,
    ) -> Result<Node<'core>, Error> {
        match self {
            FilterMode::PointBob => point_bob(core, api, src),
            FilterMode::Spline36Bob => spline36_bob(core, api, src, true),
            FilterMode::Nnedi3 => nnedi3(core, api, src, 3, false),
            FilterMode::Nnedi3CL => nnedi3(core, api, src, 3, true),
        }
    }
}

pub(crate) fn point_bob<'core>(
    core: CoreRef<'core>,
    api: API,
    src: &Node<'core>,
) -> Result<Node<'core>, Error> {
    let clip = separate_rows(core, api, src)?;
    let res = if let Constant(res) = clip.info().resolution {
        res
    } else {
        bail!("Resolution is not constant");
    };
    point_resize(core, api, &clip, res.width as i64, 2 * res.height as i64)
}

pub(crate) fn spline36_bob<'core>(
    core: CoreRef<'core>,
    api: API,
    src: &Node<'core>,
    process_chroma: bool,
) -> Result<Node<'core>, Error> {
    let clip = separate_rows(
        core,
        api,
        &convert(core, api, src, PresetFormat::Gray8 as i64)?,
    )?;
    let res = if let Constant(res) = clip.info().resolution {
        res
    } else {
        bail!("Resolution is not constant");
    };

    let even = spline36_resize_crop(
        core,
        api,
        &select_even(core, api, &clip)?,
        res.width as i64,
        2 * res.height as i64,
        0.0,
        0.25,
        res.width as f64,
        res.height as f64,
    )?;
    let odd = spline36_resize_crop(
        core,
        api,
        &select_odd(core, api, &clip)?,
        res.width as i64,
        2 * res.height as i64,
        0.0,
        -0.25,
        res.width as f64,
        res.height as f64,
    )?;
    let clip = interleave(core, api, &[&even, &odd])?;

    let format = if let Constant(format) = src.info().format {
        format
    } else {
        bail!("Resolution is not constant");
    };
    if format.id() == FormatID::from(PresetFormat::Gray8) {
        return Ok(clip);
    }

    Ok(if process_chroma {
        shuffle_planes(
            core,
            api,
            &[
                &clip,
                &spline36_bob(core, api, &u_to_y8(core, api, &src)?, false)?,
                &spline36_bob(core, api, &v_to_y8(core, api, &src)?, false)?,
            ],
            &[0, 0, 0],
            ColorFamily::YUV,
        )?
    } else {
        shuffle_planes(
            core,
            api,
            &[
                &clip,
                &select_every(core, api, &u_to_y8(core, api, &src)?, 1, &[0, 0])?,
                &select_every(core, api, &v_to_y8(core, api, &src)?, 1, &[0, 0])?,
            ],
            &[0, 0, 0],
            ColorFamily::YUV,
        )?
    })
}
