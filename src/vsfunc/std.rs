use failure::bail;
use failure::Error;
use vapoursynth::core::CoreRef;
use vapoursynth::prelude::*;

const STD_NAMESPACE: &str = "com.vapoursynth.std";

pub fn separate_rows<'core>(
    core: CoreRef<'core>,
    api: API,
    clip: FrameRef<'core>,
) -> Result<FrameRef<'core>, Error> {
    let std = core
        .get_plugin_by_id(STD_NAMESPACE)
        .map_err(Error::from)?
        .unwrap();

    let mut args = OwnedMap::new(api);
    args.set_frame("clip", &*clip);
    args.set_int("tff", 1);
    let result = std.invoke("SeparateFields", &args).map_err(Error::from)?;
    if let Some(e) = result.error() {
        bail!("{}", e);
    }
    let clip = result.get_frame("clip").map_err(Error::from)?;

    let mut args = OwnedMap::new(api);
    args.set_frame("clip", &*clip);
    args.set_int("value", 0);
    let result = std.invoke("SetFieldBased", &args).map_err(Error::from)?;
    if let Some(e) = result.error() {
        bail!("{}", e);
    }
    result.get_frame("clip").map_err(Error::from)
}

pub fn blur_v<'core>(
    core: CoreRef<'core>,
    api: API,
    clip: FrameRef<'core>,
    kernel: &[i64],
) -> Result<FrameRef<'core>, Error> {
    let std = core
        .get_plugin_by_id(STD_NAMESPACE)
        .map_err(Error::from)?
        .unwrap();

    let mut args = OwnedMap::new(api);
    args.set_frame("clip", &*clip);
    args.set_int_array("matrix", kernel);
    args.set_data("mode", "v".as_bytes());
    let result = std.invoke("Convolution", &args).map_err(Error::from)?;
    if let Some(e) = result.error() {
        bail!("{}", e);
    }
    result.get_frame("clip").map_err(Error::from)
}

pub fn select_even<'core>(
    core: CoreRef<'core>,
    api: API,
    clip: FrameRef<'core>,
) -> Result<FrameRef<'core>, Error> {
    select_every(core, api, clip, 2, &[0])
}

pub fn select_odd<'core>(
    core: CoreRef<'core>,
    api: API,
    clip: FrameRef<'core>,
) -> Result<FrameRef<'core>, Error> {
    select_every(core, api, clip, 2, &[1])
}

pub fn select_every<'core>(
    core: CoreRef<'core>,
    api: API,
    clip: FrameRef<'core>,
    cycle: i64,
    offsets: &[i64],
) -> Result<FrameRef<'core>, Error> {
    let std = core
        .get_plugin_by_id(STD_NAMESPACE)
        .map_err(Error::from)?
        .unwrap();

    let mut args = OwnedMap::new(api);
    args.set_frame("clip", &*clip);
    args.set_int("cycle", cycle);
    args.set_int_array("offsets", offsets);
    let result = std.invoke("SelectEvery", &args).map_err(Error::from)?;
    if let Some(e) = result.error() {
        bail!("{}", e);
    }
    result.get_frame("clip").map_err(Error::from)
}

pub fn interleave<'core>(
    core: CoreRef<'core>,
    api: API,
    clips: &[FrameRef<'core>],
) -> Result<FrameRef<'core>, Error> {
    let std = core
        .get_plugin_by_id(STD_NAMESPACE)
        .map_err(Error::from)?
        .unwrap();

    let mut args = OwnedMap::new(api);
    for clip in clips {
        args.append_frame("clips", &*clip);
    }
    let result = std.invoke("Interleave", &args).map_err(Error::from)?;
    if let Some(e) = result.error() {
        bail!("{}", e);
    }
    result.get_frame("clip").map_err(Error::from)
}

pub fn shuffle_planes<'core>(
    core: CoreRef<'core>,
    api: API,
    clips: &[FrameRef<'core>],
    planes: &[i64],
    color_family: ColorFamily,
) -> Result<FrameRef<'core>, Error> {
    let std = core
        .get_plugin_by_id(STD_NAMESPACE)
        .map_err(Error::from)?
        .unwrap();

    let mut args = OwnedMap::new(api);
    for clip in clips {
        args.append_frame("clips", &*clip);
    }
    args.set_int_array("planes", planes);
    args.set_int("colorfamily", color_family as i64);
    let result = std.invoke("ShufflePlanes", &args).map_err(Error::from)?;
    if let Some(e) = result.error() {
        bail!("{}", e);
    }
    result.get_frame("clip").map_err(Error::from)
}
