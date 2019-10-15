use failure::bail;
use failure::Error;
use vapoursynth::core::CoreRef;
use vapoursynth::prelude::*;

const YADIFMOD_NAMESPACE: &str = "com.holywu.yadifmod";

pub(crate) fn yadifmod<'core>(
    core: CoreRef<'core>,
    api: API,
    clip: &FrameRef<'core>,
    edeint: &FrameRef<'core>,
    order: i64,
    mode: i64,
) -> Result<FrameRef<'core>, Error> {
    let nnedi = core
        .get_plugin_by_id(YADIFMOD_NAMESPACE)
        .map_err(Error::from)?
        .unwrap();

    let mut args = OwnedMap::new(api);
    args.set_frame("clip", &*clip);
    args.set_frame("edeint", &*edeint);
    args.set_int("order", order);
    args.set_int("mode", mode);
    let result = nnedi.invoke("Yadifmod", &args).map_err(Error::from)?;
    if let Some(e) = result.error() {
        bail!("{}", e);
    }
    result.get_frame("clip").map_err(Error::from)
}
