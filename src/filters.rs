use crate::lutxy::lutxy_diff;
use crate::util::{max, median3, min};
use crate::MPEG2STINX_NAMESPACE;
use failure::Error;
use failure::{bail, format_err};
use vapoursynth::api::API;
use vapoursynth::core::CoreRef;
use vapoursynth::frame::FrameRef;
use vapoursynth::map::OwnedMap;
use vapoursynth::node::Node;
use vapoursynth::plugins::*;
use vapoursynth::video_info::VideoInfo;

pub(crate) struct Min<'core> {
    pub clip1: Node<'core>,
    pub clip2: Node<'core>,
}

impl<'core> Filter<'core> for Min<'core> {
    fn video_info(&self, _api: API, _core: CoreRef<'core>) -> Vec<VideoInfo<'core>> {
        vec![self.clip1.info()]
    }

    fn get_frame_initial(
        &self,
        _api: API,
        _core: CoreRef<'core>,
        context: FrameContext,
        n: usize,
    ) -> Result<Option<FrameRef<'core>>, Error> {
        self.clip1.request_frame_filter(context, n);
        self.clip2.request_frame_filter(context, n);
        Ok(None)
    }

    fn get_frame(
        &self,
        _api: API,
        core: CoreRef<'core>,
        context: FrameContext,
        n: usize,
    ) -> Result<FrameRef<'core>, Error> {
        let clip1 = self
            .clip1
            .get_frame_filter(context, n)
            .ok_or_else(|| format_err!("Min: Couldn't get clip1 frame"))?;
        let clip2 = self
            .clip2
            .get_frame_filter(context, n)
            .ok_or_else(|| format_err!("Min: Couldn't get clip2 frame"))?;

        min(core, &clip1, &clip2)
    }
}

pub(crate) fn min_clip<'core>(
    core: CoreRef<'core>,
    api: API,
    clip1: &Node<'core>,
    clip2: &Node<'core>,
) -> Result<Node<'core>, Error> {
    let mpeg2stinx = core
        .get_plugin_by_id(MPEG2STINX_NAMESPACE)
        .map_err(Error::from)?
        .unwrap();

    let mut args = OwnedMap::new(api);
    args.set_node("clip1", &*clip1)?;
    args.set_node("clip2", &*clip2)?;
    let result = mpeg2stinx.invoke("Min", &args).map_err(Error::from)?;
    if let Some(e) = result.error() {
        bail!("{}", e);
    }
    result.get_node("clip").map_err(Error::from)
}

pub(crate) struct Max<'core> {
    pub clip1: Node<'core>,
    pub clip2: Node<'core>,
}

impl<'core> Filter<'core> for Max<'core> {
    fn video_info(&self, _api: API, _core: CoreRef<'core>) -> Vec<VideoInfo<'core>> {
        vec![self.clip1.info()]
    }

    fn get_frame_initial(
        &self,
        _api: API,
        _core: CoreRef<'core>,
        context: FrameContext,
        n: usize,
    ) -> Result<Option<FrameRef<'core>>, Error> {
        self.clip1.request_frame_filter(context, n);
        self.clip2.request_frame_filter(context, n);
        Ok(None)
    }

    fn get_frame(
        &self,
        _api: API,
        core: CoreRef<'core>,
        context: FrameContext,
        n: usize,
    ) -> Result<FrameRef<'core>, Error> {
        let clip1 = self
            .clip1
            .get_frame_filter(context, n)
            .ok_or_else(|| format_err!("Max: Couldn't get clip1 frame"))?;
        let clip2 = self
            .clip2
            .get_frame_filter(context, n)
            .ok_or_else(|| format_err!("Max: Couldn't get clip2 frame"))?;

        max(core, &clip1, &clip2)
    }
}

pub(crate) fn max_clip<'core>(
    core: CoreRef<'core>,
    api: API,
    clip1: &Node<'core>,
    clip2: &Node<'core>,
) -> Result<Node<'core>, Error> {
    let mpeg2stinx = core
        .get_plugin_by_id(MPEG2STINX_NAMESPACE)
        .map_err(Error::from)?
        .unwrap();

    let mut args = OwnedMap::new(api);
    args.set_node("clip1", &*clip1)?;
    args.set_node("clip2", &*clip2)?;
    let result = mpeg2stinx.invoke("Max", &args).map_err(Error::from)?;
    if let Some(e) = result.error() {
        bail!("{}", e);
    }
    result.get_node("clip").map_err(Error::from)
}

pub(crate) struct Median3<'core> {
    pub clip1: Node<'core>,
    pub clip2: Node<'core>,
    pub clip3: Node<'core>,
    pub process_chroma: bool,
}

impl<'core> Filter<'core> for Median3<'core> {
    fn video_info(&self, _api: API, _core: CoreRef<'core>) -> Vec<VideoInfo<'core>> {
        vec![self.clip1.info()]
    }

    fn get_frame_initial(
        &self,
        _api: API,
        _core: CoreRef<'core>,
        context: FrameContext,
        n: usize,
    ) -> Result<Option<FrameRef<'core>>, Error> {
        self.clip1.request_frame_filter(context, n);
        self.clip2.request_frame_filter(context, n);
        self.clip3.request_frame_filter(context, n);
        Ok(None)
    }

    fn get_frame(
        &self,
        _api: API,
        core: CoreRef<'core>,
        context: FrameContext,
        n: usize,
    ) -> Result<FrameRef<'core>, Error> {
        let clip1 = self
            .clip1
            .get_frame_filter(context, n)
            .ok_or_else(|| format_err!("Max: Couldn't get clip1 frame"))?;
        let clip2 = self
            .clip2
            .get_frame_filter(context, n)
            .ok_or_else(|| format_err!("Max: Couldn't get clip2 frame"))?;
        let clip3 = self
            .clip3
            .get_frame_filter(context, n)
            .ok_or_else(|| format_err!("Max: Couldn't get clip2 frame"))?;

        median3(core, &clip1, &clip2, &clip3, self.process_chroma)
    }
}

pub(crate) fn median3_clip<'core>(
    core: CoreRef<'core>,
    api: API,
    clip1: &Node<'core>,
    clip2: &Node<'core>,
    clip3: &Node<'core>,
    process_chroma: bool,
) -> Result<Node<'core>, Error> {
    let mpeg2stinx = core
        .get_plugin_by_id(MPEG2STINX_NAMESPACE)
        .map_err(Error::from)?
        .unwrap();

    let mut args = OwnedMap::new(api);
    args.set_node("clip1", &*clip1)?;
    args.set_node("clip2", &*clip2)?;
    args.set_node("clip3", &*clip3)?;
    args.set_int("process_chroma", process_chroma as i64)?;
    let result = mpeg2stinx.invoke("Median3", &args).map_err(Error::from)?;
    if let Some(e) = result.error() {
        bail!("{}", e);
    }
    result.get_node("clip").map_err(Error::from)
}

pub(crate) struct LutXYDiff<'core> {
    pub clip1: Node<'core>,
    pub clip2: Node<'core>,
}

impl<'core> Filter<'core> for LutXYDiff<'core> {
    fn video_info(&self, _api: API, _core: CoreRef<'core>) -> Vec<VideoInfo<'core>> {
        vec![self.clip1.info()]
    }

    fn get_frame_initial(
        &self,
        _api: API,
        _core: CoreRef<'core>,
        context: FrameContext,
        n: usize,
    ) -> Result<Option<FrameRef<'core>>, Error> {
        self.clip1.request_frame_filter(context, n);
        self.clip2.request_frame_filter(context, n);
        Ok(None)
    }

    fn get_frame(
        &self,
        _api: API,
        core: CoreRef<'core>,
        context: FrameContext,
        n: usize,
    ) -> Result<FrameRef<'core>, Error> {
        let clip1 = self
            .clip1
            .get_frame_filter(context, n)
            .ok_or_else(|| format_err!("LutXYDiff: Couldn't get clip1 frame"))?;
        let clip2 = self
            .clip2
            .get_frame_filter(context, n)
            .ok_or_else(|| format_err!("LutXYDiff: Couldn't get clip2 frame"))?;

        lutxy_diff(core, &clip1, &clip2)
    }
}

pub(crate) fn lutxy_diff_clip<'core>(
    core: CoreRef<'core>,
    api: API,
    clip1: &Node<'core>,
    clip2: &Node<'core>,
) -> Result<Node<'core>, Error> {
    let mpeg2stinx = core
        .get_plugin_by_id(MPEG2STINX_NAMESPACE)
        .map_err(Error::from)?
        .unwrap();

    let mut args = OwnedMap::new(api);
    args.set_node("clip1", &*clip1)?;
    args.set_node("clip2", &*clip2)?;
    let result = mpeg2stinx.invoke("LutXYDiff", &args).map_err(Error::from)?;
    if let Some(e) = result.error() {
        bail!("{}", e);
    }
    result.get_node("clip").map_err(Error::from)
}
