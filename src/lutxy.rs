use super::*;
use failure::Error;
use vapoursynth::core::CoreRef;
use vapoursynth::prelude::*;

// Equivalent AVS:
// `mt_lutxy(x,y,expr="x x y - "+string(sstr)+" * +",y=3,u=3,v=3)`
pub(crate) fn lutxy_sharp<'core>(
    core: CoreRef<'core>,
    clip1: &FrameRef<'core>,
    clip2: &FrameRef<'core>,
    strength: f32,
) -> Result<FrameRef<'core>, Error> {
    let mut filtered = FrameRefMut::copy_of(core, &*clip1);

    // Assume formats are equivalent, because this is an internal function
    let plane_count = clip1.format().plane_count();
    let bytes_per_sample = clip1.format().bytesPerSample;
    for plane in 0..plane_count {
        match bytes_per_sample {
            1 => sharp_loop_u8(clip1, clip2, &mut filtered, plane, strength)?,
            2 => sharp_loop_u16(clip1, clip2, &mut filtered, plane, strength)?,
            4 => sharp_loop_u32(clip1, clip2, &mut filtered, plane, strength)?,
            _ => unreachable!(),
        }
    }
    Ok(FrameRef::from(filtered))
}

macro_rules! sharp_fn {
    ($pix_ty:ty, $math_ty:ty) => {
        paste::item! {
            fn [<sharp_loop_ $pix_ty>]<'core>(
                clip1: &FrameRef<'core>,
                clip2: &FrameRef<'core>,
                filtered: &mut FrameRefMut<'core>,
                plane: usize,
                strength: f32,
            ) -> Result<(), Error> {
                let bit_depth = clip1.format().bitsPerSample;
                let max_pix_val = (1 << bit_depth) - 1;
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
                    let xf = x as f32;
                    let yf = y as f32;
                    *target = clamp(
                        x as $math_ty + ((xf - yf) * strength) as $math_ty,
                        0, max_pix_val
                    ) as $pix_ty;
                }
                Ok(())
            }
        }
    };
}
sharp_fn!(u8, i32);
sharp_fn!(u16, i64);
sharp_fn!(u32, i64);

// Equivalent AVS:
// `mt_lutxy(x,y,expr="x y - "+string(sstr)+" * 128 +",y=3,u=3,v=3)`
// and also fixed to work with high bit depth
pub(crate) fn lutxy_sharpd<'core>(
    core: CoreRef<'core>,
    clip1: &FrameRef<'core>,
    clip2: &FrameRef<'core>,
    strength: f32,
) -> Result<FrameRef<'core>, Error> {
    let mut filtered = FrameRefMut::copy_of(core, &*clip1);

    // Assume formats are equivalent, because this is an internal function
    let plane_count = clip1.format().plane_count();
    let bytes_per_sample = clip1.format().bytesPerSample;
    for plane in 0..plane_count {
        match bytes_per_sample {
            1 => sharpd_loop_u8(clip1, clip2, &mut filtered, plane, strength)?,
            2 => sharpd_loop_u16(clip1, clip2, &mut filtered, plane, strength)?,
            4 => sharpd_loop_u32(clip1, clip2, &mut filtered, plane, strength)?,
            _ => unreachable!(),
        }
    }
    Ok(FrameRef::from(filtered))
}

macro_rules! sharpd_fn {
    ($pix_ty:ty, $math_ty:ty) => {
        paste::item! {
            fn [<sharpd_loop_ $pix_ty>]<'core>(
                clip1: &FrameRef<'core>,
                clip2: &FrameRef<'core>,
                filtered: &mut FrameRefMut<'core>,
                plane: usize,
                strength: f32,
            ) -> Result<(), Error> {
                let bit_depth = clip1.format().bitsPerSample;
                let max_pix_val = (1 << bit_depth) - 1;
                let half_val = 1 << (bit_depth / 2);
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
                    let x = x as f32;
                    let y = y as f32;
                    *target = clamp(
                        half_val + ((x - y) * strength) as $math_ty,
                        0, max_pix_val
                    ) as $pix_ty;
                }
                Ok(())
            }
        }
    };
}
sharpd_fn!(u8, i32);
sharpd_fn!(u16, i64);
sharpd_fn!(u32, i64);

// Equivalent AVS:
// `mt_lutxy(x,y,expr="x 128 - y 128 - * 0 < "+string(scl)+" 1 ? x 128 - abs y 128 - abs < x y ? 128 - * 128 +",y=3,u=3,v=3)`
// and also fixed to work with high bit depth
pub(crate) fn lutxy_limd<'core>(
    core: CoreRef<'core>,
    clip1: &FrameRef<'core>,
    clip2: &FrameRef<'core>,
    scale: f32,
) -> Result<FrameRef<'core>, Error> {
    let mut filtered = FrameRefMut::copy_of(core, &*clip1);

    // Assume formats are equivalent, because this is an internal function
    let plane_count = clip1.format().plane_count();
    let bytes_per_sample = clip1.format().bytesPerSample;
    for plane in 0..plane_count {
        match bytes_per_sample {
            1 => limd_loop_u8(clip1, clip2, &mut filtered, plane, scale)?,
            2 => limd_loop_u16(clip1, clip2, &mut filtered, plane, scale)?,
            4 => limd_loop_u32(clip1, clip2, &mut filtered, plane, scale)?,
            _ => unreachable!(),
        }
    }
    Ok(FrameRef::from(filtered))
}

macro_rules! limd_fn {
    ($pix_ty:ty, $math_ty:ty) => {
        paste::item! {
            fn [<limd_loop_ $pix_ty>]<'core>(
                clip1: &FrameRef<'core>,
                clip2: &FrameRef<'core>,
                filtered: &mut FrameRefMut<'core>,
                plane: usize,
                scale: f32,
            ) -> Result<(), Error> {
                let bit_depth = clip1.format().bitsPerSample;
                let max_pix_val = (1 << bit_depth) - 1;
                let half_val = 1 << (bit_depth / 2);
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
                    let x = x as $math_ty;
                    let y = y as $math_ty;
                    *target = clamp(
                        half_val + (
                            ((if (x - half_val).abs() < (y - half_val).abs() { x } else { y }) - half_val) as f32
                            * (if (x - half_val) * (y - half_val) < 0 { scale } else { 1.0 })
                        ) as $math_ty,
                        0, max_pix_val
                    ) as $pix_ty;
                }
                Ok(())
            }
        }
    };
}
limd_fn!(u8, i32);
limd_fn!(u16, i64);
limd_fn!(u32, i64);

// Equivalent AVS:
// `mt_lutxy(x,y,expr="x y - abs",y=3,u=3,v=3)`
pub(crate) fn lutxy_diff<'core>(
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
            1 => diff_loop_u8(clip1, clip2, &mut filtered, plane)?,
            2 => diff_loop_u16(clip1, clip2, &mut filtered, plane)?,
            4 => diff_loop_u32(clip1, clip2, &mut filtered, plane)?,
            _ => unreachable!(),
        }
    }
    Ok(FrameRef::from(filtered))
}

macro_rules! diff_fn {
    ($pix_ty:ty, $math_ty:ty) => {
        paste::item! {
            fn [<diff_loop_ $pix_ty>]<'core>(
                clip1: &FrameRef<'core>,
                clip2: &FrameRef<'core>,
                filtered: &mut FrameRefMut<'core>,
                plane: usize,
            ) -> Result<(), Error> {
                let bit_depth = clip1.format().bitsPerSample;
                let max_pix_val = (1 << bit_depth) - 1;
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
                    *target = clamp(
                        (x as $math_ty - y as $math_ty).abs(),
                        0, max_pix_val
                    ) as $pix_ty;
                }
                Ok(())
            }
        }
    };
}
diff_fn!(u8, i16);
diff_fn!(u16, i32);
diff_fn!(u32, i64);

// Equivalent AVS:
// `mt_lutxy(x,y,expr="x y - 128 +",y=3,u=3,v=3)`
pub(crate) fn make_diff<'core>(
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
            1 => make_diff_loop_u8(clip1, clip2, &mut filtered, plane)?,
            2 => make_diff_loop_u16(clip1, clip2, &mut filtered, plane)?,
            4 => make_diff_loop_u32(clip1, clip2, &mut filtered, plane)?,
            _ => unreachable!(),
        }
    }
    Ok(FrameRef::from(filtered))
}

macro_rules! make_diff_fn {
    ($pix_ty:ty, $math_ty:ty) => {
        paste::item! {
            fn [<make_diff_loop_ $pix_ty>]<'core>(
                clip1: &FrameRef<'core>,
                clip2: &FrameRef<'core>,
                filtered: &mut FrameRefMut<'core>,
                plane: usize,
            ) -> Result<(), Error> {
                let bit_depth = clip1.format().bitsPerSample;
                let max_pix_val = (1 << bit_depth) - 1;
                let half_val = 1 << (bit_depth / 2);
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
                    *target = clamp(
                        (x as $math_ty - y as $math_ty) + half_val,
                        0, max_pix_val
                    ) as $pix_ty;
                }
                Ok(())
            }
        }
    };
}
make_diff_fn!(u8, i16);
make_diff_fn!(u16, i32);
make_diff_fn!(u32, i64);

// Equivalent AVS:
// `mt_lutxy(x,y,expr="x y + 128 -",y=3,u=3,v=3)`
pub(crate) fn add_diff<'core>(
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
            1 => add_diff_loop_u8(clip1, clip2, &mut filtered, plane)?,
            2 => add_diff_loop_u16(clip1, clip2, &mut filtered, plane)?,
            4 => add_diff_loop_u32(clip1, clip2, &mut filtered, plane)?,
            _ => unreachable!(),
        }
    }
    Ok(FrameRef::from(filtered))
}

macro_rules! add_diff_fn {
    ($pix_ty:ty, $math_ty:ty) => {
        paste::item! {
            fn [<add_diff_loop_ $pix_ty>]<'core>(
                clip1: &FrameRef<'core>,
                clip2: &FrameRef<'core>,
                filtered: &mut FrameRefMut<'core>,
                plane: usize,
            ) -> Result<(), Error> {
                let bit_depth = clip1.format().bitsPerSample;
                let max_pix_val = (1 << bit_depth) - 1;
                let half_val = 1 << (bit_depth / 2);
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
                    *target = clamp(
                        (x as $math_ty + y as $math_ty) - half_val,
                        0, max_pix_val
                    ) as $pix_ty;
                }
                Ok(())
            }
        }
    };
}
add_diff_fn!(u8, i16);
add_diff_fn!(u16, i32);
add_diff_fn!(u32, i64);
