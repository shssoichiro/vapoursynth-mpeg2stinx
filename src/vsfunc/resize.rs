use failure::Error;
use failure::{bail, format_err};
use vapoursynth::core::CoreRef;
use vapoursynth::prelude::*;

const RESIZE_NAMESPACE: &str = "com.vapoursynth.resize";

pub(crate) fn point_resize<'core>(
    core: CoreRef<'core>,
    api: API,
    clip: &Node<'core>,
    width: i64,
    height: i64,
) -> Result<Node<'core>, Error> {
    let resize = core
        .get_plugin_by_id(RESIZE_NAMESPACE)
        .map_err(Error::from)?
        .ok_or_else(|| format_err!("resize namespace not found"))?;

    let mut args = OwnedMap::new(api);
    args.set_node("clip", &*clip)?;
    args.set_int("width", width)?;
    args.set_int("height", height)?;
    let result = resize.invoke("Point", &args).map_err(Error::from)?;
    if let Some(e) = result.error() {
        bail!("{}", e);
    }
    result.get_node("clip").map_err(Error::from)
}

pub(crate) fn bilinear_resize<'core>(
    core: CoreRef<'core>,
    api: API,
    clip: &Node<'core>,
    width: i64,
    height: i64,
) -> Result<Node<'core>, Error> {
    let resize = core
        .get_plugin_by_id(RESIZE_NAMESPACE)
        .map_err(Error::from)?
        .ok_or_else(|| format_err!("resize namespace not found"))?;

    let mut args = OwnedMap::new(api);
    args.set_node("clip", &*clip)?;
    args.set_int("width", width)?;
    args.set_int("height", height)?;
    let result = resize.invoke("Bilinear", &args).map_err(Error::from)?;
    if let Some(e) = result.error() {
        bail!("{}", e);
    }
    result.get_node("clip").map_err(Error::from)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn spline36_resize_crop<'core>(
    core: CoreRef<'core>,
    api: API,
    clip: &Node<'core>,
    width: i64,
    height: i64,
    src_left: f64,
    src_top: f64,
    src_width: f64,
    src_height: f64,
) -> Result<Node<'core>, Error> {
    let resize = core
        .get_plugin_by_id(RESIZE_NAMESPACE)
        .map_err(Error::from)?
        .ok_or_else(|| format_err!("resize namespace not found"))?;

    let mut args = OwnedMap::new(api);
    args.set_node("clip", &*clip)?;
    args.set_int("width", width)?;
    args.set_int("height", height)?;
    args.set_float("src_left", src_left)?;
    args.set_float("src_top", src_top)?;
    args.set_float("src_width", src_width)?;
    args.set_float("src_height", src_height)?;
    let result = resize.invoke("Spline36", &args).map_err(Error::from)?;
    if let Some(e) = result.error() {
        bail!("{}", e);
    }
    result.get_node("clip").map_err(Error::from)
}

pub(crate) fn convert<'core>(
    core: CoreRef<'core>,
    api: API,
    clip: &Node<'core>,
    format: PresetFormat,
) -> Result<Node<'core>, Error> {
    let resize = core
        .get_plugin_by_id(RESIZE_NAMESPACE)
        .map_err(Error::from)?
        .ok_or_else(|| format_err!("resize namespace not found"))?;

    let mut args = OwnedMap::new(api);
    args.set_node("clip", &*clip)?;
    args.set_int("format", format as i64)?;
    let result = resize.invoke("Spline36", &args).map_err(Error::from)?;
    if let Some(e) = result.error() {
        bail!("{}", e);
    }
    result.get_node("clip").map_err(Error::from)
}
