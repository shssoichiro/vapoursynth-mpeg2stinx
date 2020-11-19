use failure::bail;
use failure::Error;
use vapoursynth::core::CoreRef;
use vapoursynth::prelude::*;

const MISC_NAMESPACE: &str = "com.vapoursynth.misc";

pub(crate) fn average_frames<'core>(
    core: CoreRef<'core>,
    api: API,
    clips: &[&Node<'core>],
    weights: Option<&[f64]>,
) -> Result<Node<'core>, Error> {
    let misc = core
        .get_plugin_by_id(MISC_NAMESPACE)
        .map_err(Error::from)?
        .unwrap();

    let mut args = OwnedMap::new(api);
    if let Some(weights) = weights {
        if weights.len() != clips.len() {
            bail!("Number of clips must equal number of weights");
        }
        for (clip, weight) in clips.iter().zip(weights) {
            args.append_node("clips", &*clip)?;
            args.append_float("weights", *weight)?;
        }
    } else {
        for clip in clips {
            args.append_node("clips", &*clip)?;
            args.append_float("weights", 1.0)?;
        }
    }
    let result = misc.invoke("AverageFrames", &args).map_err(Error::from)?;
    if let Some(e) = result.error() {
        bail!("{}", e);
    }
    result.get_node("clip").map_err(Error::from)
}
