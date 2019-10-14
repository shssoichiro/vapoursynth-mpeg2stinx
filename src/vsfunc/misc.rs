use failure::bail;
use failure::Error;
use vapoursynth::core::CoreRef;
use vapoursynth::prelude::*;

const MISC_NAMESPACE: &str = "com.vapoursynth.misc";

pub fn average_frames<'core>(
    core: CoreRef<'core>,
    api: API,
    clips: &[FrameRef<'core>],
) -> Result<FrameRef<'core>, Error> {
    let misc = core
        .get_plugin_by_id(MISC_NAMESPACE)
        .map_err(Error::from)?
        .unwrap();

    let mut args = OwnedMap::new(api);
    for clip in clips {
        args.append_frame("clips", &*clip);
        args.append_int("weights", 1);
    }
    let result = misc.invoke("AverageFrames", &args).map_err(Error::from)?;
    if let Some(e) = result.error() {
        bail!("{}", e);
    }
    result.get_frame("clip").map_err(Error::from)
}
