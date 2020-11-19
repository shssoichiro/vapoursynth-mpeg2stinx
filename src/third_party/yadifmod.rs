use failure::Error;
use failure::{bail, format_err};
use vapoursynth::core::CoreRef;
use vapoursynth::prelude::*;

const YADIFMOD_NAMESPACE: &str = "com.holywu.yadifmod";

pub(crate) fn yadifmod<'core>(
    core: CoreRef<'core>,
    api: API,
    clip: &Node<'core>,
    edeint: &Node<'core>,
    order: i64,
    mode: i64,
) -> Result<Node<'core>, Error> {
    let nnedi = core
        .get_plugin_by_id(YADIFMOD_NAMESPACE)
        .map_err(Error::from)?
        .ok_or_else(|| format_err!("yadifmod plugin not found"))?;

    let mut args = OwnedMap::new(api);
    args.set_node("clip", &*clip)?;
    args.set_node("edeint", &*edeint)?;
    args.set_int("order", order)?;
    args.set_int("mode", mode)?;
    let result = nnedi.invoke("Yadifmod", &args).map_err(Error::from)?;
    if let Some(e) = result.error() {
        bail!("{}", e);
    }
    result.get_node("clip").map_err(Error::from)
}
