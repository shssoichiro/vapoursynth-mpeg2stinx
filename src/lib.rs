#![warn(clippy::all)]

mod deint;
mod filters;
mod lutxy;
mod repair;
mod third_party;
mod util;
mod vsfunc;

use self::deint::*;
use self::filters::*;
use self::lutxy::*;
use self::repair::*;
use self::third_party::*;
use self::util::*;
use self::vsfunc::*;
use failure::ensure;
use failure::format_err;
use failure::Error;
use std::convert::TryFrom;
use vapoursynth::core::CoreRef;
use vapoursynth::export_vapoursynth_plugin;
use vapoursynth::format::FormatID;
use vapoursynth::make_filter_function;
use vapoursynth::plugins::*;
use vapoursynth::prelude::*;
use vapoursynth::video_info::VideoInfo;

const MPEG2STINX_NAMESPACE: &str = "com.soichiro.mpeg2stinx";

struct Mpeg2Stinx<'core> {
    src: Node<'core>,
    nuked: Node<'core>,
    nuked_blurred: Option<Node<'core>>,
    contra: bool,
    sstr: f32,
    scl: f32,
}

impl<'core> Filter<'core> for Mpeg2Stinx<'core> {
    fn video_info(&self, _api: API, _core: CoreRef<'core>) -> Vec<VideoInfo<'core>> {
        vec![self.src.info()]
    }

    fn get_frame_initial(
        &self,
        _api: API,
        _core: CoreRef<'core>,
        context: FrameContext,
        n: usize,
    ) -> Result<Option<FrameRef<'core>>, Error> {
        self.src.request_frame_filter(context, n);
        self.nuked.request_frame_filter(context, n);
        if let Some(ref nuked_blurred) = self.nuked_blurred {
            nuked_blurred.request_frame_filter(context, n);
        }
        Ok(None)
    }

    fn get_frame(
        &self,
        _api: API,
        core: CoreRef<'core>,
        context: FrameContext,
        n: usize,
    ) -> Result<FrameRef<'core>, Error> {
        let nuked = self
            .nuked
            .get_frame_filter(context, n)
            .ok_or_else(|| format_err!("Mpeg2Stinx: Couldn't get the nuked frame"))?;

        if !self.contra {
            return Ok(nuked);
        }

        let src = self
            .src
            .get_frame_filter(context, n)
            .ok_or_else(|| format_err!("Mpeg2Stinx: Couldn't get the source frame"))?;
        let nuked_blurred = self
            .nuked_blurred
            .as_ref()
            .unwrap()
            .get_frame_filter(context, n)
            .ok_or_else(|| format_err!("Mpeg2Stinx: Couldn't get the nuked blurred frame"))?;

        let sharp = lutxy_sharp(core, &nuked, &nuked_blurred, self.sstr)
            .map_err(|e| e.context("Mpeg2Stinx: "))?;

        if self.scl == 0.0 {
            return Ok(
                median3(core, &nuked, &sharp, &src, true).map_err(|e| e.context("Mpeg2Stinx: "))?
            );
        }

        let nukedd = make_diff(core, &src, &nuked).map_err(|e| e.context("Mpeg2Stinx: "))?;
        let sharpd = lutxy_sharpd(core, &nuked, &nuked_blurred, self.sstr)
            .map_err(|e| e.context("Mpeg2Stinx: "))?;
        let limd =
            lutxy_limd(core, &sharpd, &nukedd, self.scl).map_err(|e| e.context("Mpeg2Stinx: "))?;
        Ok(add_diff(core, &nuked, &limd).map_err(|e| e.context("Mpeg2Stinx: "))?)
    }
}

make_filter_function! {
    Mpeg2StinxFunction, "Mpeg2Stinx"

    #[allow(clippy::too_many_arguments)]
    fn create_mpeg2stinx<'core>(
        api: API,
        core: CoreRef<'core>,
        clip: Node<'core>,
        mode: Option<i64>,
        sw: Option<i64>,
        sh: Option<i64>,
        contra: Option<i64>,
        blurv: Option<f64>,
        sstr: Option<f64>,
        scl: Option<f64>,
        dither: Option<i64>,
        order: Option<i64>,
        diffscl: Option<f64>,
    ) -> Result<Option<Box<dyn Filter<'core> + 'core>>, Error> {
        let mode = match mode {
            Some(mode) => FilterMode::try_from(mode)?,
            None => FilterMode::default()
        };
        let sw = match sw {
            Some(sw) => {
                ensure!(
                    sw >= 0,
                    "Mpeg2Stinx: sw and sh must both be non-negative integers"
                );
                sw as u32
            }
            None => 1
        };
        let sh = match sh {
            Some(sh) => {
                ensure!(
                    sh >= 0,
                    "Mpeg2Stinx: sw and sh must both be non-negative integers"
                );
                sh as u32
            }
            None => 1
        };
        let contra = contra.map(|contra| contra != 0).unwrap_or(true);
        let blurv = blurv.unwrap_or_else(|| if contra { 0.9 } else { 0.0 });
        let sstr = sstr.unwrap_or(2.0);
        let scl = scl.unwrap_or(0.25);
        let dither = dither.map(|dither| dither != 0).unwrap_or(false);
        let order = order.unwrap_or(-1);
        ensure!(
            order >= -1 && order <= 1,
            "Mpeg2Stinx: order must be -1, 0 or 1"
        );
        if let Some(diffscl) = diffscl {
            ensure!(
                diffscl >= 0.0,
                "Mpeg2Stinx: diffscl must be a non-negative number"
            );
        }

        let a = cross_field_repair2(
            core,
            api,
            &clip,
            Some(
                &deint(core, api, &clip, mode, order)
                    .map_err(|e| e.context("Mpeg2Stinx: "))?,
            ),
            sw,
            sh,
            true,
        )
        .map_err(|e| e.context("Mpeg2Stinx: "))?;
        let a = if let Some(diffscl) = diffscl {
            temp_limit(core, api, &clip, &a, &clip, diffscl).map_err(|e| e.context("Mpeg2Stinx: "))?
        } else {
            a
        };

        let b = cross_field_repair2(
            core,
            api,
            &a,
            Some(
                &deint(core, api, &a, mode, order)
                    .map_err(|e| e.context("Mpeg2Stinx: "))?,
            ),
            sw,
            sh,
            true,
        )
        .map_err(|e| e.context("Mpeg2Stinx: "))?;
        let b = if let Some(diffscl) = diffscl {
            temp_limit(core, api, &a, &b, &clip, diffscl).map_err(|e| e.context("Mpeg2Stinx: "))?
        } else {
            b
        };

        let average = average(core, api, &a, &b, dither).map_err(|e| e.context("Mpeg2Stinx: "))?;

        let nuked = if blurv > 0.0 {
            crate::util::blur_v(core, api, &average, blurv)
                .map_err(|e| e.context("Mpeg2Stinx: "))?
        } else {
            average
        };
        let nuked_blurred = if contra {
            Some(crate::util::blur_v(
                core,
                api,
                &crate::util::blur_v(core, api, &nuked, 1.0).map_err(|e| e.context("Mpeg2Stinx: "))?,
                1.0,
            )
            .map_err(|e| e.context("Mpeg2Stinx: "))?)
        } else {
            None
        };

        Ok(Some(Box::new(Mpeg2Stinx {
            src: clip,
            nuked,
            nuked_blurred,
            contra,
            sstr: sstr as f32,
            scl: scl as f32,
        })))
    }
}

make_filter_function! {
    MinFunction, "Min"

    fn create_min<'core>(
        api: API,
        core: CoreRef<'core>,
        clip1: Node<'core>,
        clip2: Node<'core>,
    ) -> Result<Option<Box<dyn Filter<'core> + 'core>>, Error> {
        Ok(Some(Box::new(Min {
            clip1,
            clip2,
        })))
    }
}

make_filter_function! {
    MaxFunction, "Max"

    fn create_max<'core>(
        api: API,
        core: CoreRef<'core>,
        clip1: Node<'core>,
        clip2: Node<'core>,
    ) -> Result<Option<Box<dyn Filter<'core> + 'core>>, Error> {
        Ok(Some(Box::new(Max {
            clip1,
            clip2,
        })))
    }
}

make_filter_function! {
    Median3Function, "Median3"

    fn create_median3<'core>(
        api: API,
        core: CoreRef<'core>,
        clip1: Node<'core>,
        clip2: Node<'core>,
        clip3: Node<'core>,
        process_chroma: Option<i64>,
    ) -> Result<Option<Box<dyn Filter<'core> + 'core>>, Error> {
        Ok(Some(Box::new(Median3 {
            clip1,
            clip2,
            clip3,
            process_chroma: process_chroma.unwrap_or(0) > 0
        })))
    }
}

make_filter_function! {
    LutXYDiffFunction, "LutXYDiff"

    fn create_lutxy_diff<'core>(
        api: API,
        core: CoreRef<'core>,
        clip1: Node<'core>,
        clip2: Node<'core>,
    ) -> Result<Option<Box<dyn Filter<'core> + 'core>>, Error> {
        Ok(Some(Box::new(LutXYDiff {
            clip1,
            clip2,
        })))
    }
}

export_vapoursynth_plugin! {
    Metadata {
        identifier: MPEG2STINX_NAMESPACE,
        namespace: "mpeg2stinx",
        name: "Mpeg2Stinx",
        read_only: true,
    },
    [Mpeg2StinxFunction::new(), MinFunction::new(), MaxFunction::new(), LutXYDiffFunction::new()]
}
