use crate::{
    div, Active, Anonymous, AnyElement, BorrowWindow, Bounds, Click, Div, DivState, Element,
    ElementId, ElementIdentity, EventListeners, Hover, Identified, Interactive, IntoAnyElement,
    LayoutId, NonFocusable, Pixels, SharedString, StyleRefinement, Styled, ViewContext,
};
use futures::FutureExt;
use util::ResultExt;

pub struct Img<V: 'static + Send + Sync, K: ElementIdentity = Anonymous> {
    base: Div<K, NonFocusable, V>,
    uri: Option<SharedString>,
    grayscale: bool,
}

pub fn img<V>() -> Img<V, Anonymous>
where
    V: 'static + Send + Sync,
{
    Img {
        base: div(),
        uri: None,
        grayscale: false,
    }
}

impl<V, K> Img<V, K>
where
    V: 'static + Send + Sync,
    K: ElementIdentity,
{
    pub fn uri(mut self, uri: impl Into<SharedString>) -> Self {
        self.uri = Some(uri.into());
        self
    }

    pub fn grayscale(mut self, grayscale: bool) -> Self {
        self.grayscale = grayscale;
        self
    }
}

impl<V: 'static + Send + Sync> Img<V, Anonymous> {
    pub fn id(self, id: impl Into<ElementId>) -> Img<V, Identified> {
        Img {
            base: self.base.id(id),
            uri: self.uri,
            grayscale: self.grayscale,
        }
    }
}

impl<V, K> IntoAnyElement<V> for Img<V, K>
where
    V: 'static + Send + Sync,
    K: ElementIdentity,
{
    fn into_any(self) -> AnyElement<V> {
        AnyElement::new(self)
    }
}

impl<V, K> Element for Img<V, K>
where
    V: Send + Sync + 'static,
    K: ElementIdentity,
{
    type ViewState = V;
    type ElementState = DivState;

    fn id(&self) -> Option<crate::ElementId> {
        self.base.id()
    }

    fn initialize(
        &mut self,
        view_state: &mut V,
        element_state: Option<Self::ElementState>,
        cx: &mut ViewContext<V>,
    ) -> Self::ElementState {
        self.base.initialize(view_state, element_state, cx)
    }

    fn layout(
        &mut self,
        view_state: &mut V,
        element_state: &mut Self::ElementState,
        cx: &mut ViewContext<Self::ViewState>,
    ) -> LayoutId {
        self.base.layout(view_state, element_state, cx)
    }

    fn paint(
        &mut self,
        bounds: Bounds<Pixels>,
        view: &mut V,
        element_state: &mut Self::ElementState,
        cx: &mut ViewContext<V>,
    ) {
        cx.stack(0, |cx| {
            self.base.paint(bounds, view, element_state, cx);
        });

        let style = self.base.compute_style(bounds, element_state, cx);
        let corner_radii = style.corner_radii;

        if let Some(uri) = self.uri.clone() {
            let image_future = cx.image_cache.get(uri);
            if let Some(data) = image_future
                .clone()
                .now_or_never()
                .and_then(ResultExt::log_err)
            {
                let corner_radii = corner_radii.to_pixels(bounds.size, cx.rem_size());
                cx.stack(1, |cx| {
                    cx.paint_image(bounds, corner_radii, data, self.grayscale)
                        .log_err()
                });
            } else {
                cx.spawn(|_, mut cx| async move {
                    if image_future.await.log_err().is_some() {
                        cx.on_next_frame(|cx| cx.notify());
                    }
                })
                .detach()
            }
        }
    }
}

impl<V, K> Styled for Img<V, K>
where
    V: 'static + Send + Sync,
    K: ElementIdentity,
{
    fn style(&mut self) -> &mut StyleRefinement {
        self.base.style()
    }
}

impl<V, K> Interactive for Img<V, K>
where
    V: 'static + Send + Sync,
    K: ElementIdentity,
{
    fn listeners(&mut self) -> &mut EventListeners<V> {
        self.base.listeners()
    }
}

impl<V, K> Hover for Img<V, K>
where
    V: 'static + Send + Sync,
    K: ElementIdentity,
{
    fn set_hover_style(&mut self, group: Option<SharedString>, style: StyleRefinement) {
        self.base.set_hover_style(group, style);
    }
}

impl<V> Click for Img<V, Identified> where V: 'static + Send + Sync {}

impl<V> Active for Img<V, Identified>
where
    V: 'static + Send + Sync,
{
    fn set_active_style(&mut self, group: Option<SharedString>, style: StyleRefinement) {
        self.base.set_active_style(group, style)
    }
}