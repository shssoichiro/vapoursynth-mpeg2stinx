use failure::bail;
use failure::Error;
use vapoursynth::core::CoreRef;
use vapoursynth::prelude::*;

const RGVS_NAMESPACE: &str = "com.vapoursynth.rgvs";

pub(crate) fn clense<'core>(
    core: &'core CoreRef<'core>,
    api: API,
    clip: &FrameRef<'core>,
    previous: &FrameRef<'core>,
    next: &FrameRef<'core>,
    planes: &[i64],
) -> Result<FrameRef<'core>, Error> {
    let rgvs = core
        .get_plugin_by_id(RGVS_NAMESPACE)
        .map_err(Error::from)?
        .unwrap();

    let mut args = OwnedMap::new(api);
    args.set_frame("clip", &*clip);
    args.set_frame("previous", &*previous);
    args.set_frame("next", &*next);
    args.set_int_array("planes", planes);
    let result = rgvs.invoke("Clense", &args).map_err(Error::from)?;
    if let Some(e) = result.error() {
        bail!("{}", e);
    }
    result.get_frame("clip").map_err(Error::from)
}

pub(crate) fn repair<'core>(
    core: &'core CoreRef<'core>,
    api: API,
    clip: &FrameRef<'core>,
    repair_clip: &FrameRef<'core>,
    mode: i64,
) -> Result<FrameRef<'core>, Error> {
    let rgvs = core
        .get_plugin_by_id(RGVS_NAMESPACE)
        .map_err(Error::from)?
        .unwrap();

    let mut args = OwnedMap::new(api);
    args.set_frame("clip", &*clip);
    args.set_frame("repair_clip", &*repair_clip);
    args.set_int("mode", mode);
    let result = rgvs.invoke("Repair", &args).map_err(Error::from)?;
    if let Some(e) = result.error() {
        bail!("{}", e);
    }
    result.get_frame("clip").map_err(Error::from)
}
