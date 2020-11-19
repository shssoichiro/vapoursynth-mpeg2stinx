use crate::util::ExpandMode;
use failure::Error;
use failure::{bail, format_err};
use vapoursynth::core::CoreRef;
use vapoursynth::prelude::*;

const STD_NAMESPACE: &str = "com.vapoursynth.std";

pub(crate) fn separate_rows<'core>(
    core: CoreRef<'core>,
    api: API,
    clip: &Node<'core>,
) -> Result<Node<'core>, Error> {
    let std = core
        .get_plugin_by_id(STD_NAMESPACE)
        .map_err(Error::from)?
        .ok_or_else(|| format_err!("std namespace not found"))?;

    let mut args = OwnedMap::new(api);
    args.set_node("clip", clip)?;
    args.set_int("tff", 1)?;
    let result = std.invoke("SeparateFields", &args).map_err(Error::from)?;
    if let Some(e) = result.error() {
        bail!("{}", e);
    }
    let clip = result.get_node("clip").map_err(Error::from)?;

    let mut args = OwnedMap::new(api);
    args.set_node("clip", &clip)?;
    args.set_int("value", 0)?;
    let result = std.invoke("SetFieldBased", &args).map_err(Error::from)?;
    if let Some(e) = result.error() {
        bail!("{}", e);
    }
    result.get_node("clip").map_err(Error::from)
}

pub(crate) fn weave_rows<'core>(
    core: CoreRef<'core>,
    api: API,
    clip: &Node<'core>,
) -> Result<Node<'core>, Error> {
    let std = core
        .get_plugin_by_id(STD_NAMESPACE)
        .map_err(Error::from)?
        .ok_or_else(|| format_err!("std namespace not found"))?;

    let mut args = OwnedMap::new(api);
    args.set_node("clip", clip)?;
    args.set_int("tff", 1)?;
    let result = std.invoke("DoubleWeave", &args).map_err(Error::from)?;
    if let Some(e) = result.error() {
        bail!("{}", e);
    }
    let clip = result.get_node("clip").map_err(Error::from)?;

    select_even(core, api, &clip)
}

pub(crate) fn blur_v<'core>(
    core: CoreRef<'core>,
    api: API,
    clip: &Node<'core>,
    kernel: &[i64],
) -> Result<Node<'core>, Error> {
    let std = core
        .get_plugin_by_id(STD_NAMESPACE)
        .map_err(Error::from)?
        .ok_or_else(|| format_err!("std namespace not found"))?;

    let mut args = OwnedMap::new(api);
    args.set_node("clip", clip)?;
    args.set_int_array("matrix", kernel)?;
    args.set_data("mode", b"v")?;
    let result = std.invoke("Convolution", &args).map_err(Error::from)?;
    if let Some(e) = result.error() {
        bail!("{}", e);
    }
    result.get_node("clip").map_err(Error::from)
}

pub(crate) fn select_even<'core>(
    core: CoreRef<'core>,
    api: API,
    clip: &Node<'core>,
) -> Result<Node<'core>, Error> {
    select_every(core, api, clip, 2, &[0])
}

pub(crate) fn select_odd<'core>(
    core: CoreRef<'core>,
    api: API,
    clip: &Node<'core>,
) -> Result<Node<'core>, Error> {
    select_every(core, api, clip, 2, &[1])
}

pub(crate) fn select_every<'core>(
    core: CoreRef<'core>,
    api: API,
    clip: &Node<'core>,
    cycle: i64,
    offsets: &[i64],
) -> Result<Node<'core>, Error> {
    let std = core
        .get_plugin_by_id(STD_NAMESPACE)
        .map_err(Error::from)?
        .ok_or_else(|| format_err!("std namespace not found"))?;

    let mut args = OwnedMap::new(api);
    args.set_node("clip", clip)?;
    args.set_int("cycle", cycle)?;
    args.set_int_array("offsets", offsets)?;
    let result = std.invoke("SelectEvery", &args).map_err(Error::from)?;
    if let Some(e) = result.error() {
        bail!("{}", e);
    }
    result.get_node("clip").map_err(Error::from)
}

pub(crate) fn interleave<'core>(
    core: CoreRef<'core>,
    api: API,
    clips: &[&Node<'core>],
) -> Result<Node<'core>, Error> {
    let std = core
        .get_plugin_by_id(STD_NAMESPACE)
        .map_err(Error::from)?
        .ok_or_else(|| format_err!("std namespace not found"))?;

    let mut args = OwnedMap::new(api);
    for clip in clips {
        args.append_node("clips", clip)?;
    }
    let result = std.invoke("Interleave", &args).map_err(Error::from)?;
    if let Some(e) = result.error() {
        bail!("{}", e);
    }
    result.get_node("clip").map_err(Error::from)
}

pub(crate) fn shuffle_planes<'core>(
    core: CoreRef<'core>,
    api: API,
    clips: &[&Node<'core>],
    planes: &[i64],
    color_family: ColorFamily,
) -> Result<Node<'core>, Error> {
    let std = core
        .get_plugin_by_id(STD_NAMESPACE)
        .map_err(Error::from)?
        .ok_or_else(|| format_err!("std namespace not found"))?;

    let mut args = OwnedMap::new(api);
    for clip in clips {
        args.append_node("clips", clip)?;
    }
    args.set_int_array("planes", planes)?;
    args.set_int("colorfamily", color_family as i64)?;
    let result = std.invoke("ShufflePlanes", &args).map_err(Error::from)?;
    if let Some(e) = result.error() {
        bail!("{}", e);
    }
    result.get_node("clip").map_err(Error::from)
}

pub(crate) fn expand<'core>(
    core: CoreRef<'core>,
    api: API,
    clip: &Node<'core>,
    mode: ExpandMode,
    process_chroma: bool,
) -> Result<Node<'core>, Error> {
    if mode == ExpandMode::None {
        return Ok(clip.clone());
    }

    let std = core
        .get_plugin_by_id(STD_NAMESPACE)
        .map_err(Error::from)?
        .ok_or_else(|| format_err!("std namespace not found"))?;

    let planes: &[i64] = if process_chroma { &[0, 1, 2] } else { &[0] };
    let mut args = OwnedMap::new(api);
    args.set_node("clip", clip)?;
    args.set_int_array("coordinates", &mode.to_coords())?;
    args.set_int_array("planes", planes)?;
    let result = std.invoke("Maximum", &args).map_err(Error::from)?;
    if let Some(e) = result.error() {
        bail!("{}", e);
    }
    result.get_node("clip").map_err(Error::from)
}

pub(crate) fn inpand<'core>(
    core: CoreRef<'core>,
    api: API,
    clip: &Node<'core>,
    mode: ExpandMode,
    process_chroma: bool,
) -> Result<Node<'core>, Error> {
    if mode == ExpandMode::None {
        return Ok(clip.clone());
    }

    let std = core
        .get_plugin_by_id(STD_NAMESPACE)
        .map_err(Error::from)?
        .ok_or_else(|| format_err!("std namespace not found"))?;

    let planes: &[i64] = if process_chroma { &[0, 1, 2] } else { &[0] };
    let mut args = OwnedMap::new(api);
    args.set_node("clip", clip)?;
    args.set_int_array("coordinates", &mode.to_coords())?;
    args.set_int_array("planes", planes)?;
    let result = std.invoke("Minimum", &args).map_err(Error::from)?;
    if let Some(e) = result.error() {
        bail!("{}", e);
    }
    result.get_node("clip").map_err(Error::from)
}
