use super::*;
use failure::{bail, ensure, Error};
use std::cmp;
use vapoursynth::core::CoreRef;
use vapoursynth::prelude::*;
use vapoursynth::video_info::Property::Constant;

pub(crate) fn u_to_y8<'core>(
    core: CoreRef<'core>,
    api: API,
    src: &Node<'core>,
) -> Result<Node<'core>, Error> {
    if let Constant(format) = src.info().format {
        if format.plane_count() == 1 {
            return convert(core, api, src, PresetFormat::Gray8 as i32);
        }
    } else {
        bail!("Format is not constant");
    };
    convert(
        core,
        api,
        &shuffle_planes(core, api, &[src], &[1], ColorFamily::Gray)?,
        PresetFormat::Gray8 as i32,
    )
}

pub(crate) fn v_to_y8<'core>(
    core: CoreRef<'core>,
    api: API,
    src: &Node<'core>,
) -> Result<Node<'core>, Error> {
    if let Constant(format) = src.info().format {
        if format.plane_count() == 1 {
            return convert(core, api, src, PresetFormat::Gray8 as i32);
        }
    } else {
        bail!("Format is not constant");
    };
    convert(
        core,
        api,
        &shuffle_planes(core, api, &[src], &[2], ColorFamily::Gray)?,
        PresetFormat::Gray8 as i32,
    )
}

/// max of the Y/U/V planes, resizing if necessary
#[allow(clippy::many_single_char_names)]
pub(crate) fn max_yuv<'core>(
    core: CoreRef<'core>,
    api: API,
    src: &Node<'core>,
) -> Result<Node<'core>, Error> {
    let y = convert(core, api, src, PresetFormat::Gray8 as i32)?;
    let u = u_to_y8(core, api, src)?;
    let v = v_to_y8(core, api, src)?;
    let y_res = if let Constant(res) = y.info().resolution {
        res
    } else {
        bail!("Luma resolution is not constant");
    };
    let u_res = if let Constant(res) = u.info().resolution {
        res
    } else {
        bail!("Chroma resolution is not constant");
    };
    let w = y_res.width as i64;
    let h = y_res.height as i64;
    let u_w = u_res.width as i64;
    let u_h = u_res.height as i64;

    let yc = bilinear_resize(core, api, &y, u_w, u_h)?;
    let ls = max_clip(
        core,
        api,
        &max_clip(core, api, &y, &bilinear_resize(core, api, &u, w, h)?)?,
        &bilinear_resize(core, api, &v, w, h)?,
    )?;
    let cs = max_clip(core, api, &max_clip(core, api, &yc, &u)?, &v)?;
    shuffle_planes(core, api, &[&ls, &cs, &cs], &[0, 0, 0], ColorFamily::YUV)
}

pub(crate) fn median3<'core>(
    core: CoreRef<'core>,
    clip1: &FrameRef<'core>,
    clip2: &FrameRef<'core>,
    clip3: &FrameRef<'core>,
    process_chroma: bool,
) -> Result<FrameRef<'core>, Error> {
    let mut filtered = FrameRefMut::copy_of(core, &*clip1);

    ensure!(
        clip1.format().plane_count() == clip2.format().plane_count(),
        "Clip2 had {} planes, expected {}",
        clip2.format().plane_count(),
        clip1.format().plane_count()
    );
    ensure!(
        clip1.format().plane_count() == clip3.format().plane_count(),
        "Clip3 had {} planes, expected {}",
        clip3.format().plane_count(),
        clip1.format().plane_count()
    );
    ensure!(
        clip1.format().bits_per_sample() == clip2.format().bits_per_sample(),
        "Clip2 had bit depth of {}, expected {}",
        clip2.format().bits_per_sample(),
        clip1.format().bits_per_sample()
    );
    ensure!(
        clip1.format().bits_per_sample() == clip3.format().bits_per_sample(),
        "Clip3 had bit depth of {}, expected {}",
        clip3.format().bits_per_sample(),
        clip1.format().bits_per_sample()
    );
    let plane_count = cmp::min(
        clip1.format().plane_count(),
        if process_chroma { 3 } else { 1 },
    );
    let bytes_per_sample = clip1.format().bytesPerSample;
    for plane in 0..plane_count {
        match bytes_per_sample {
            1 => median_loop_u8(clip1, clip2, clip3, &mut filtered, plane)?,
            2 => median_loop_u16(clip1, clip2, clip3, &mut filtered, plane)?,
            4 => median_loop_u32(clip1, clip2, clip3, &mut filtered, plane)?,
            _ => unreachable!(),
        }
    }
    Ok(FrameRef::from(filtered))
}

macro_rules! median_fn {
    ($pix_ty:ty) => {
        paste::item! {
            fn [<median_loop_ $pix_ty>]<'core>(
                clip1: &FrameRef<'core>,
                clip2: &FrameRef<'core>,
                clip3: &FrameRef<'core>,
                filtered: &mut FrameRefMut<'core>,
                plane: usize,
            ) -> Result<(), Error> {
                for (((&x, &y), &z), target) in clip1
                    .plane::<$pix_ty>(plane)
                    .map_err(Error::from)?
                    .into_iter()
                    .zip(
                        clip2
                            .plane::<$pix_ty>(plane)
                            .map_err(Error::from)?
                            .into_iter(),
                    ).zip(
                        clip3
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
                    *target = if x > y {
                        if y > z { y }
                        else if x > z { z }
                        else { x }
                    } else {
                        if x > z { x }
                        else if y > z { z }
                        else { y }
                    };
                }
                Ok(())
            }
        }
    };
}
median_fn!(u8);
median_fn!(u16);
median_fn!(u32);

pub(crate) fn temp_limit<'core>(
    core: CoreRef<'core>,
    api: API,
    clip: &Node<'core>,
    flt: &Node<'core>,
    reff: &Node<'core>,
    diffscl: f64,
) -> Result<Node<'core>, Error> {
    let adj = select_every(core, api, reff, 1, &[-1, 1])?;
    let diff = max_yuv(
        core,
        api,
        &separate_rows(
            core,
            api,
            &lutxy_diff_clip(core, api, &select_every(core, api, clip, 1, &[0, 0])?, &adj)?,
        )?,
    )?;
    let diff2 = weave_rows(
        core,
        api,
        &expand_multi(
            core,
            api,
            &min_clip(
                core,
                api,
                &select_every(core, api, &diff, 4, &[0, 1])?,
                &select_every(core, api, &diff, 4, &[2, 3])?,
            )?,
            2,
            1,
            true,
        )?,
    )?;
    let a = average_frames(core, api, &[clip, &diff2], Some(&[1.0, -diffscl]))?;
    let b = average_frames(core, api, &[clip, &diff2], Some(&[1.0, diffscl]))?;
    median3_clip(core, api, &a, &b, flt, true)
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
    clip: &Node<'core>,
    sw: u32,
    sh: u32,
    process_chroma: bool,
) -> Result<Node<'core>, Error> {
    let mode = match (sw, sh) {
        (0, 0) => ExpandMode::None,
        (0, _) => ExpandMode::Vertical,
        (_, 0) => ExpandMode::Horizontal,
        _ => ExpandMode::Square,
    };

    if mode == ExpandMode::None {
        return Ok(clip.clone());
    }

    let expanded = expand(core, api, clip, mode, process_chroma)?;
    expand_multi(
        core,
        api,
        &expanded,
        sw.saturating_sub(1),
        sh.saturating_sub(1),
        process_chroma,
    )
}

pub(crate) fn inpand_multi<'core>(
    core: CoreRef<'core>,
    api: API,
    clip: &Node<'core>,
    sw: u32,
    sh: u32,
    process_chroma: bool,
) -> Result<Node<'core>, Error> {
    let mode = match (sw, sh) {
        (0, 0) => ExpandMode::None,
        (0, _) => ExpandMode::Vertical,
        (_, 0) => ExpandMode::Horizontal,
        _ => ExpandMode::Square,
    };

    if mode == ExpandMode::None {
        return Ok(clip.clone());
    }

    let inpanded = inpand(core, api, clip, mode, process_chroma)?;
    inpand_multi(
        core,
        api,
        &inpanded,
        sw.saturating_sub(1),
        sh.saturating_sub(1),
        process_chroma,
    )
}

pub(crate) fn min<'core>(
    core: CoreRef<'core>,
    clip1: &FrameRef<'core>,
    clip2: &FrameRef<'core>,
) -> Result<FrameRef<'core>, Error> {
    let mut filtered = FrameRefMut::copy_of(core, &*clip1);

    // Assume formats are equivalent, because this is an internal function
    let plane_count = clip1.format().plane_count();
    let bytes_per_sample = clip1.format().bytesPerSample;
    for plane in 0..plane_count {
        match bytes_per_sample {
            1 => min_loop_u8(clip1, clip2, &mut filtered, plane)?,
            2 => min_loop_u16(clip1, clip2, &mut filtered, plane)?,
            4 => min_loop_u32(clip1, clip2, &mut filtered, plane)?,
            _ => unreachable!(),
        }
    }
    Ok(FrameRef::from(filtered))
}

macro_rules! min_fn {
    ($pix_ty:ty) => {
        paste::item! {
            fn [<min_loop_ $pix_ty>]<'core>(
                clip1: &FrameRef<'core>,
                clip2: &FrameRef<'core>,
                filtered: &mut FrameRefMut<'core>,
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
    clip1: &FrameRef<'core>,
    clip2: &FrameRef<'core>,
) -> Result<FrameRef<'core>, Error> {
    let mut filtered = FrameRefMut::copy_of(core, &*clip1);

    // Assume formats are equivalent, because this is an internal function
    let plane_count = clip1.format().plane_count();
    let bytes_per_sample = clip1.format().bytesPerSample;
    for plane in 0..plane_count {
        match bytes_per_sample {
            1 => max_loop_u8(clip1, clip2, &mut filtered, plane)?,
            2 => max_loop_u16(clip1, clip2, &mut filtered, plane)?,
            4 => max_loop_u32(clip1, clip2, &mut filtered, plane)?,
            _ => unreachable!(),
        }
    }
    Ok(FrameRef::from(filtered))
}

macro_rules! max_fn {
    ($pix_ty:ty) => {
        paste::item! {
            fn [<max_loop_ $pix_ty>]<'core>(
                clip1: &FrameRef<'core>,
                clip2: &FrameRef<'core>,
                filtered: &mut FrameRefMut<'core>,
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

pub(crate) fn build_blurv_kernel(strength: f64) -> [f64; 3] {
    // Vapoursynth's Convolution kernel will round our numbers to integers,
    // so scale up as far as possible for the most accuracy.
    const MAX: f64 = 1023.0;
    let inner_factor = 1.0 / 2f64.powf(strength);
    let outer_factor = (1.0 - 1.0 / 2f64.powf(strength)) / 2.0;
    let inner = if strength > 0.0 {
        inner_factor * MAX
    } else {
        MAX / inner_factor
    };
    let outer = if strength > 0.0 {
        outer_factor * MAX
    } else {
        MAX / outer_factor
    };
    [outer, inner, outer]
}

pub(crate) fn blur_v<'core>(
    core: CoreRef<'core>,
    api: API,
    src: &Node<'core>,
    strength: f64,
) -> Result<Node<'core>, Error> {
    let kernel = if (strength - 1.0).abs() < std::f64::EPSILON {
        build_blurv_kernel(1.0)
    } else {
        build_blurv_kernel(strength)
    };
    crate::vsfunc::blur_v(core, api, src, &kernel)
}

pub(crate) fn deint<'core>(
    core: CoreRef<'core>,
    api: API,
    src: &Node<'core>,
    mode: FilterMode,
    order: i64,
) -> Result<Node<'core>, Error> {
    let bobbed = mode.deint(core, api, src)?;
    Ok(match order {
        -1 => bobbed,
        0 => select_every(
            core,
            api,
            // use mode=3 because src is nominally progressive and the spatial
            // check does more harm than good on progressive things. it's also
            // faster.
            &yadifmod(
                core,
                api,
                src,
                &select_every(core, api, &bobbed, 2, &[1, 0])?,
                0,
                3,
            )?,
            2,
            &[1, 0],
        )?,
        1 => yadifmod(core, api, src, &bobbed, 1, 3)?,
        _ => unreachable!(),
    })
}

pub(crate) fn average<'core>(
    core: CoreRef<'core>,
    api: API,
    a: &Node<'core>,
    b: &Node<'core>,
    dither: bool,
) -> Result<Node<'core>, Error> {
    if dither {
        // dither_post(r_average_w(a, 0.5, b, 0.5, true)?, 7)
        todo!()
    } else {
        average_frames(core, api, &[a, b], None)
    }
}
