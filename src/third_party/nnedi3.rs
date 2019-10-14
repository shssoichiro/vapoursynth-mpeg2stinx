use failure::bail;
use failure::Error;
use vapoursynth::core::CoreRef;
use vapoursynth::prelude::*;

const NNEDI3_NAMESPACE: &str = "com.deinterlace.nnedi3";
const NNEDI3CL_NAMESPACE: &str = "com.holywu.nnedi3cl";

pub fn nnedi3<'core>(
    core: CoreRef<'core>,
    api: API,
    clip: FrameRef<'core>,
    field: i64,
    opencl: bool,
) -> Result<FrameRef<'core>, Error> {
    let nnedi = core
        .get_plugin_by_id(if opencl {
            NNEDI3CL_NAMESPACE
        } else {
            NNEDI3_NAMESPACE
        })
        .map_err(Error::from)?
        .unwrap();
    let fn_name = if opencl { "NNEDI3CL" } else { "nnedi3" };

    let mut args = OwnedMap::new(api);
    args.set_frame("clip", &*clip);
    args.set_int("field", field);
    let result = nnedi.invoke(fn_name, &args).map_err(Error::from)?;
    if let Some(e) = result.error() {
        bail!("{}", e);
    }
    result.get_frame("clip").map_err(Error::from)
}
