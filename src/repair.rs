use super::*;
use failure::{bail, Error};
use vapoursynth::core::CoreRef;
use vapoursynth::prelude::*;
use vapoursynth::video_info::Property::Constant;

pub(crate) fn cross_field_repair2<'core>(
    core: CoreRef<'core>,
    api: API,
    src: &Node<'core>,
    bobbed: Option<&Node<'core>>,
    sw: u32,
    sh: u32,
    process_chroma: bool,
) -> Result<Node<'core>, Error> {
    let format = if let Constant(format) = src.info().format {
        format.id()
    } else {
        bail!("Format not constant");
    };

    let bobbed = convert(
        core,
        api,
        &match bobbed {
            Some(bobbed) => bobbed.clone(),
            None => spline36_bob(core, api, src, process_chroma)?,
        },
        format.into(),
    )?;
    let (re, ro) = if sw == 1 && sh == 1 {
        let re = repair(
            core,
            api,
            src,
            &convert(core, api, &select_even(core, api, &bobbed)?, format.into())?,
            1,
        )?;
        let ro = repair(
            core,
            api,
            src,
            &convert(core, api, &select_odd(core, api, &bobbed)?, format.into())?,
            1,
        )?;
        (re, ro)
    } else {
        let bobbed_ex = expand_multi(core, api, &bobbed, sw, sh, process_chroma)?;
        let bobbed_in = inpand_multi(core, api, &bobbed, sw, sh, process_chroma)?;
        let re = median3_clip(
            core,
            api,
            src,
            &select_even(core, api, &bobbed_ex)?,
            &select_even(core, api, &bobbed_in)?,
            process_chroma,
        )?;
        let ro = median3_clip(
            core,
            api,
            src,
            &select_odd(core, api, &bobbed_ex)?,
            &select_odd(core, api, &bobbed_in)?,
            process_chroma,
        )?;
        (re, ro)
    };
    let clip = interleave(core, api, &[&re, &ro])?;
    let clip = separate_rows(core, api, &clip)?;
    let clip = select_every(core, api, &clip, 4, &[2, 1])?;
    weave_rows(core, api, &clip)
}
