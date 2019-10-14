use super::*;
use failure::Error;
use vapoursynth::core::CoreRef;
use vapoursynth::prelude::*;

pub fn u_to_y8<'core>(
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

pub fn v_to_y8<'core>(
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

#[inline(always)]
pub fn clamp<T: PartialOrd>(input: T, min: T, max: T) -> T {
    if input < min {
        min
    } else if input > max {
        max
    } else {
        input
    }
}
