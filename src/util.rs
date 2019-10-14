use super::*;
use failure::Error;
use vapoursynth::core::CoreRef;
use vapoursynth::prelude::*;

pub(crate) fn u_to_y8<'core>(
    core: CoreRef<'core>,
    api: API,
    src: FrameRef<'core>,
) -> Result<FrameRef<'core>, Error> {
    convert(
        core,
        api,
        shuffle_planes(core, api, &[src], &[1], ColorFamily::Gray)?,
        PresetFormat::Gray8,
    )
}

pub(crate) fn v_to_y8<'core>(
    core: CoreRef<'core>,
    api: API,
    src: FrameRef<'core>,
) -> Result<FrameRef<'core>, Error> {
    convert(
        core,
        api,
        shuffle_planes(core, api, &[src], &[2], ColorFamily::Gray)?,
        PresetFormat::Gray8,
    )
}

/// max of the Y/U/V planes, resizing if necessary
#[allow(clippy::many_single_char_names)]
pub(crate) fn max_yuv<'core>(
    core: CoreRef<'core>,
    api: API,
    src: FrameRef<'core>,
) -> Result<FrameRef<'core>, Error> {
    let y = convert(core, api, src, PresetFormat::Gray8)?;
    let u = u_to_y8(core, api, src)?;
    let v = v_to_y8(core, api, src)?;
    let w = y.width(0) as i64;
    let h = y.height(0) as i64;

    let yc = bilinear_resize(core, api, y, u.width(0) as i64, u.height(0) as i64)?;
    let ls = max(
        core,
        max(core, y, bilinear_resize(core, api, u, w, h)?)?,
        bilinear_resize(core, api, v, w, h)?,
    )?;
    let cs = max(core, max(core, yc, u)?, v)?;
    shuffle_planes(core, api, &[ls, cs, cs], &[0, 0, 0], ColorFamily::YUV)
}

pub(crate) fn median3<'core>(
    core: CoreRef<'core>,
    api: API,
    current: FrameRef<'core>,
    previous: FrameRef<'core>,
    next: FrameRef<'core>,
    process_chroma: bool,
) -> Result<FrameRef<'core>, Error> {
    let planes: &[i64] = if process_chroma { &[0, 1, 2] } else { &[0] };
    clense(core, api, current, previous, next, planes)
}

pub(crate) fn temp_limit<'core>(
    core: CoreRef<'core>,
    api: API,
    clip: FrameRef<'core>,
    flt: FrameRef<'core>,
    reff: FrameRef<'core>,
    diffscl: f64,
) -> Result<FrameRef<'core>, Error> {
    let adj = select_every(core, api, reff, -1, &[1])?;
    let diff = max_yuv(
        core,
        api,
        separate_rows(
            core,
            api,
            lutxy_diff(core, select_every(core, api, clip, 1, &[0, 0])?, adj)?,
        )?,
    )?;
    let diff2 = weave_rows(
        core,
        api,
        expand_multi(
            core,
            api,
            min(
                core,
                select_every(core, api, diff, 4, &[0, 1])?,
                select_every(core, api, diff, 4, &[2, 3])?,
            )?,
            2,
            1,
            true,
        )?,
    )?;
    let a = average_frames(core, api, &[clip, diff2], Some(&[1.0, -diffscl]))?;
    let b = average_frames(core, api, &[clip, diff2], Some(&[1.0, diffscl]))?;
    median3(core, api, a, b, flt, true)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExpandMode {
    Square,
    Horizontal,
    Vertical,
    None,
}

impl ExpandMode {
    pub(crate) fn to_coords(self) -> [i64; 8] {
        match self {
            ExpandMode::Square => [1, 1, 1, 1, 1, 1, 1, 1],
            ExpandMode::Horizontal => [0, 0, 0, 1, 1, 0, 0, 0],
            ExpandMode::Vertical => [0, 1, 0, 0, 0, 0, 1, 0],
            ExpandMode::None => [0; 8],
        }
    }
}

pub(crate) fn expand_multi<'core>(
    core: CoreRef<'core>,
    api: API,
    clip: FrameRef<'core>,
    sw: u32,
    sh: u32,
    process_chroma: bool,
) -> Result<FrameRef<'core>, Error> {
    let mode = match (sw, sh) {
        (0, 0) => ExpandMode::None,
        (0, _) => ExpandMode::Vertical,
        (_, 0) => ExpandMode::Horizontal,
        _ => ExpandMode::Square,
    };

    if mode == ExpandMode::None {
        return Ok(clip);
    }

    let expanded = expand(core, api, clip, mode, process_chroma)?;
    expand_multi(
        core,
        api,
        expanded,
        sw.saturating_sub(1),
        sh.saturating_sub(1),
        process_chroma,
    )
}

pub(crate) fn inpand_multi<'core>(
    core: CoreRef<'core>,
    api: API,
    clip: FrameRef<'core>,
    sw: u32,
    sh: u32,
    process_chroma: bool,
) -> Result<FrameRef<'core>, Error> {
    let mode = match (sw, sh) {
        (0, 0) => ExpandMode::None,
        (0, _) => ExpandMode::Vertical,
        (_, 0) => ExpandMode::Horizontal,
        _ => ExpandMode::Square,
    };

    if mode == ExpandMode::None {
        return Ok(clip);
    }

    let inpanded = inpand(core, api, clip, mode, process_chroma)?;
    inpand_multi(
        core,
        api,
        inpanded,
        sw.saturating_sub(1),
        sh.saturating_sub(1),
        process_chroma,
    )
}

pub(crate) fn min<'core>(
    core: CoreRef<'core>,
    clip1: FrameRef<'core>,
    clip2: FrameRef<'core>,
) -> Result<FrameRef<'core>, Error> {
    let mut filtered = FrameRefMut::copy_of(core, &*clip1);

    // Assume formats are equivalent, because this is an internal function
    let plane_count = clip1.format().plane_count();
    let bytes_per_sample = clip1.format().bytesPerSample;
    for plane in 0..plane_count {
        match bytes_per_sample {
            1 => min_loop_u8(clip1, clip2, filtered, plane)?,
            2 => min_loop_u16(clip1, clip2, filtered, plane)?,
            4 => min_loop_u32(clip1, clip2, filtered, plane)?,
            _ => unreachable!(),
        }
    }
    Ok(FrameRef::from(filtered))
}

macro_rules! min_fn {
    ($pix_ty:ty) => {
        paste::item! {
            fn [<min_loop_ $pix_ty>]<'core>(
                clip1: FrameRef<'core>,
                clip2: FrameRef<'core>,
                filtered: FrameRefMut<'core>,
                plane: usize,
            ) -> Result<(), Error> {
                for ((&x, &y), target) in clip1
                    .plane::<$pix_ty>(plane)
                    .map_err(Error::from)?
                    .into_iter()
                    .zip(
                        clip2
                            .plane::<$pix_ty>(plane)
                            .map_err(Error::from)?
                            .into_iter(),
                    )
                    .zip(
                        filtered
                            .plane_mut::<$pix_ty>(plane)
                            .map_err(Error::from)?
                            .iter_mut(),
                    )
                {
                    *target = ::std::cmp::min(x, y);
                }
                Ok(())
            }
        }
    };
}
min_fn!(u8);
min_fn!(u16);
min_fn!(u32);

pub(crate) fn max<'core>(
    core: CoreRef<'core>,
    clip1: FrameRef<'core>,
    clip2: FrameRef<'core>,
) -> Result<FrameRef<'core>, Error> {
    let mut filtered = FrameRefMut::copy_of(core, &*clip1);

    // Assume formats are equivalent, because this is an internal function
    let plane_count = clip1.format().plane_count();
    let bytes_per_sample = clip1.format().bytesPerSample;
    for plane in 0..plane_count {
        match bytes_per_sample {
            1 => max_loop_u8(clip1, clip2, filtered, plane)?,
            2 => max_loop_u16(clip1, clip2, filtered, plane)?,
            4 => max_loop_u32(clip1, clip2, filtered, plane)?,
            _ => unreachable!(),
        }
    }
    Ok(FrameRef::from(filtered))
}

macro_rules! max_fn {
    ($pix_ty:ty) => {
        paste::item! {
            fn [<max_loop_ $pix_ty>]<'core>(
                clip1: FrameRef<'core>,
                clip2: FrameRef<'core>,
                filtered: FrameRefMut<'core>,
                plane: usize,
            ) -> Result<(), Error> {
                for ((&x, &y), target) in clip1
                    .plane::<$pix_ty>(plane)
                    .map_err(Error::from)?
                    .into_iter()
                    .zip(
                        clip2
                            .plane::<$pix_ty>(plane)
                            .map_err(Error::from)?
                            .into_iter(),
                    )
                    .zip(
                        filtered
                            .plane_mut::<$pix_ty>(plane)
                            .map_err(Error::from)?
                            .iter_mut(),
                    )
                {
                    *target = ::std::cmp::max(x, y);
                }
                Ok(())
            }
        }
    };
}
max_fn!(u8);
max_fn!(u16);
max_fn!(u32);

#[inline(always)]
pub(crate) fn clamp<T: PartialOrd>(input: T, min: T, max: T) -> T {
    if input < min {
        min
    } else if input > max {
        max
    } else {
        input
    }
}
