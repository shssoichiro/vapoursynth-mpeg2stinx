#![warn(clippy::all)]

mod deint;
mod lutxy;
mod repair;
mod third_party;
mod util;
mod vsfunc;

use self::deint::*;
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

struct Mpeg2Stinx<'core> {
    clip: Node<'core>,
    mode: FilterMode,
    sw: u32,
    sh: u32,
    contra: bool,
    blurv: f32,
    sstr: f32,
    scl: f32,
    dither: bool,
    order: i64,
    diffscl: Option<f64>,
    // There will only be at most two different kernels,
    // so build them once to avoid doing it every frame.
    // These will be indexed as
    // `blurv_kernels[0] = kernel(str=blurv)` and
    // `blurv_kernels[1] = kernel(str=1)`.
    blurv_kernels: [[i64; 3]; 2],
}

impl<'core> Filter<'core> for Mpeg2Stinx<'core> {
    fn video_info(&self, _api: API, _core: CoreRef<'core>) -> Vec<VideoInfo<'core>> {
        vec![self.clip.info()]
    }

    fn get_frame_initial(
        &self,
        _api: API,
        _core: CoreRef<'core>,
        context: FrameContext,
        n: usize,
    ) -> Result<Option<FrameRef<'core>>, Error> {
        self.clip.request_frame_filter(context, n);
        Ok(None)
    }

    fn get_frame(
        &self,
        api: API,
        core: CoreRef<'core>,
        context: FrameContext,
        n: usize,
    ) -> Result<FrameRef<'core>, Error> {
        let src = self
            .clip
            .get_frame_filter(context, n)
            .ok_or_else(|| format_err!("MPEG2Stinx: Couldn't get the source frame"))?;

        let a = cross_field_repair2(
            core,
            api,
            src,
            Some(
                self.deint(core, api, src)
                    .map_err(|e| e.context("MPEG2Stinx"))?,
            ),
            self.sw,
            self.sh,
            true,
        )
        .map_err(|e| e.context("MPEG2Stinx"))?;
        let a = if let Some(diffscl) = self.diffscl {
            temp_limit(core, api, src, a, src, diffscl).map_err(|e| e.context("MPEG2Stinx"))?
        } else {
            a
        };
        let b = cross_field_repair2(
            core,
            api,
            a,
            Some(
                self.deint(core, api, a)
                    .map_err(|e| e.context("MPEG2Stinx"))?,
            ),
            self.sw,
            self.sh,
            true,
        )
        .map_err(|e| e.context("MPEG2Stinx"))?;
        let b = if let Some(diffscl) = self.diffscl {
            temp_limit(core, api, a, b, src, diffscl).map_err(|e| e.context("MPEG2Stinx"))?
        } else {
            b
        };
        let average = self
            .average(core, api, a, b)
            .map_err(|e| e.context("MPEG2Stinx"))?;

        let nuked = if self.blurv > 0.0 {
            self.blur_v(core, api, average, self.blurv)
                .map_err(|e| e.context("MPEG2Stinx"))?
        } else {
            average
        };
        if !self.contra {
            return Ok(nuked);
        }

        let nuked_blurred = blur_v(
            core,
            api,
            blur_v(core, api, nuked, &self.blurv_kernels[1])?,
            &self.blurv_kernels[1],
        )
        .map_err(|e| e.context("MPEG2Stinx"))?;
        let sharp = lutxy_sharp(core, nuked, nuked_blurred, self.sstr)
            .map_err(|e| e.context("MPEG2Stinx"))?;

        if self.scl == 0.0 {
            return Ok(
                median3(core, api, nuked, sharp, src, true).map_err(|e| e.context("MPEG2Stinx"))?
            );
        }

        let nukedd = mt_makediff(src, nuked).map_err(|e| e.context("MPEG2Stinx"))?;
        let sharpd = lutxy_sharpd(core, nuked, nuked_blurred, self.sstr)
            .map_err(|e| e.context("MPEG2Stinx"))?;
        let limd =
            lutxy_limd(core, sharpd, nukedd, self.scl).map_err(|e| e.context("MPEG2Stinx"))?;
        Ok(mt_adddiff(nuked, limd).map_err(|e| e.context("MPEG2Stinx"))?)
    }
}

impl<'core> Mpeg2Stinx<'core> {
    fn blur_v(
        &self,
        core: CoreRef<'core>,
        api: API,
        src: FrameRef<'core>,
        strength: f32,
    ) -> Result<FrameRef<'core>, Error> {
        let kernel = if strength == 1.0 {
            &self.blurv_kernels[1]
        } else {
            &self.blurv_kernels[0]
        };
        blur_v(core, api, src, kernel)
    }

    fn deint(
        &self,
        core: CoreRef<'core>,
        api: API,
        src: FrameRef<'core>,
    ) -> Result<FrameRef<'core>, Error> {
        let bobbed = self.mode.deint(core, api, src)?;
        Ok(match self.order {
            -1 => bobbed,
            0 => select_every(
                core,
                api,
                // use mode=3 because src is nominally progressive and the spatial
                // check does more harm than good on progressive things. it's also
                // faster.
                yadifmod(
                    core,
                    api,
                    0,
                    3,
                    select_every(core, api, bobbed, 2, &[1, 0])?,
                )?,
                2,
                &[1, 0],
            )?,
            1 => yadifmod(core, api, 1, 3, bobbed)?,
            _ => unreachable!(),
        })
    }

    fn average(
        &self,
        core: CoreRef<'core>,
        api: API,
        a: FrameRef<'core>,
        b: FrameRef<'core>,
    ) -> Result<FrameRef<'core>, Error> {
        if self.dither {
            // dither_post(r_average_w(a, 0.5, b, 0.5, true)?, 7)
            unimplemented!()
        } else {
            average_frames(core, api, &[a, b], None)
        }
    }
}

fn build_blurv_kernel(strength: f64) -> [i64; 3] {
    // Vapoursynth's Convolution kernel only accepts integers,
    // but strength accepts a float,
    // so scale up as far as possible for the most accuracy.
    const MAX: f64 = 1023.0;
    let inner_factor = 1.0 / 2f64.powf(strength);
    let outer_factor = (1.0 - 1.0 / 2f64.powf(strength)) / 2.0;
    let inner = if strength > 0.0 {
        inner_factor * MAX
    } else {
        MAX / inner_factor
    };
    let outer = if strength > 0.0 {
        outer_factor * MAX
    } else {
        MAX / outer_factor
    };
    [
        outer.round() as i64,
        inner.round() as i64,
        outer.round() as i64,
    ]
}

make_filter_function! {
    Mpeg2StinxFunction, "Mpeg2Stinx"

    #[allow(clippy::too_many_arguments)]
    fn create_mpeg2stinx<'core>(
        _api: API,
        _core: CoreRef<'core>,
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
                    "MPEG2Stinx: sw and sh must both be non-negative integers"
                );
                sw as u32
            }
            None => 1
        };
        let sh = match sh {
            Some(sh) => {
                ensure!(
                    sh >= 0,
                    "MPEG2Stinx: sw and sh must both be non-negative integers"
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
            "MPEG2Stinx: order must be -1, 0 or 1"
        );
        if let Some(diffscl) = diffscl {
            ensure!(
                diffscl >= 0.0,
                "MPEG2Stinx: diffscl must be a non-negative number"
            );
        }

        Ok(Some(Box::new(Mpeg2Stinx {
            clip,
            mode,
            sw,
            sh,
            contra,
            blurv: blurv as f32,
            sstr: sstr as f32,
            scl: scl as f32,
            dither,
            order,
            diffscl,
            blurv_kernels: [build_blurv_kernel(blurv), build_blurv_kernel(1.0)]
        })))
    }
}

export_vapoursynth_plugin! {
    Metadata {
        identifier: "com.denoise.mpeg2stinx",
        namespace: "mpeg2stinx",
        name: "MPEG2Stinx",
        read_only: true,
    },
    [Mpeg2StinxFunction::new()]
}
