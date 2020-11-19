use failure::bail;
use failure::Error;
use vapoursynth::core::CoreRef;
use vapoursynth::prelude::*;

const RGVS_NAMESPACE: &str = "com.vapoursynth.rgvs";

pub(crate) fn repair<'core>(
    core: CoreRef<'core>,
    api: API,
    clip: &Node<'core>,
    repair_clip: &Node<'core>,
    mode: i64,
) -> Result<Node<'core>, Error> {
    let rgvs = core
        .get_plugin_by_id(RGVS_NAMESPACE)
        .map_err(Error::from)?
        .unwrap();

    let mut args = OwnedMap::new(api);
    args.set_node("clip", &*clip)?;
    args.set_node("repair_clip", &*repair_clip)?;
    args.set_int("mode", mode)?;
    let result = rgvs.invoke("Repair", &args).map_err(Error::from)?;
    if let Some(e) = result.error() {
        bail!("{}", e);
    }
    result.get_node("clip").map_err(Error::from)
}
