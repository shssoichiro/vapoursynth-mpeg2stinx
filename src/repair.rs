use super::*;
use failure::Error;
use vapoursynth::core::CoreRef;
use vapoursynth::prelude::*;

pub(crate) fn cross_field_repair2<'core>(
    core: &'core CoreRef<'core>,
    api: API,
    src: &FrameRef<'core>,
    bobbed: Option<&FrameRef<'core>>,
    sw: u32,
    sh: u32,
    process_chroma: bool,
) -> Result<FrameRef<'core>, Error> {
    let bobbed = match bobbed {
        Some(bobbed) => bobbed.clone(),
        None => spline36_bob(core, api, src, process_chroma)?,
    };
    let bobbed_ex = expand_multi(core, api, &bobbed, sw, sh, process_chroma)?;
    let bobbed_in = inpand_multi(core, api, &bobbed, sw, sh, process_chroma)?;
    let re = if sw == 1 && sh == 1 {
        repair(core, api, src, &select_even(core, api, &bobbed)?, 1)?
    } else {
        median3(
            core,
            api,
            src,
            &select_even(core, api, &bobbed_ex)?,
            &select_even(core, api, &bobbed_in)?,
            process_chroma,
        )?
    };
    let ro = if sw == 1 && sh == 1 {
        repair(core, api, src, &select_odd(core, api, &bobbed)?, 1)?
    } else {
        median3(
            core,
            api,
            src,
            &select_odd(core, api, &bobbed_ex)?,
            &select_odd(core, api, &bobbed_in)?,
            process_chroma,
        )?
    };
    let clip = interleave(core, api, &[&re, &ro])?;
    let clip = separate_rows(core, api, &clip)?;
    let clip = select_every(core, api, &clip, 4, &[2, 1])?;
    weave_rows(core, api, &clip)
}
