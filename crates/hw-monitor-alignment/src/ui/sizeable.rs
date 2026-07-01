use std::rc::Rc;

use windows_core::Interface as _;
use windows_reactor::{
    Backend, ControlId, ControlKind, CustomElement, CustomElementHandle, Element, GridLength,
    HorizontalAlignment, Prop, PropValue, VerticalAlignment, grid,
};

use crate::bindings::IFrameworkElement;

/// Wraps `child` so its rendered size is reported through `on_resize`.
pub struct Sizeable {
    child: Element,
    on_resize: Option<Rc<dyn Fn(f64, f64)>>,
}

impl Sizeable {
    pub fn new<I: Into<Element>>(child: I) -> Self {
        Self {
            child: child.into(),
            on_resize: None,
        }
    }

    pub fn on_resize<F: Fn(f64, f64) + 'static>(mut self, f: F) -> Self {
        self.on_resize = Some(Rc::new(f));
        self
    }
}

pub fn sizeable<I: Into<Element>>(child: I) -> Sizeable {
    Sizeable::new(child)
}

impl From<Sizeable> for Element {
    fn from(value: Sizeable) -> Self {
        let probe: Element = SizeProbe {
            on_resize: value.on_resize,
        }
        .into();

        // Both children share the grid's single cell (row/col 0):
        // the child renders, the probe overlays it and measures the same area.
        // column by default is Star, we want Auto
        // Star is 'take remaining space', Auto is size to child
        grid([value.child, probe])
            .columns([GridLength::Auto])
            .rows([GridLength::Auto])
            .into()
    }
}

/// Invisible control.
///
/// Initially I tried to use this control to also render contents, but we cannot. Seems like `CustomElement`s can only do native rendering.
///
/// We use the `CustomElement` trait like this to get access to the native `FrameworkElement` whose `SizeChanged` event we subscribe to.
struct SizeProbe {
    on_resize: Option<Rc<dyn Fn(f64, f64)>>,
}

impl From<SizeProbe> for Element {
    fn from(value: SizeProbe) -> Self {
        Element::Custom(CustomElementHandle(Box::new(value)))
    }
}

impl CustomElement for SizeProbe {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn kind_name(&self) -> &'static str {
        "SizeProbe"
    }

    fn eq_dyn(&self, other: &dyn CustomElement) -> bool {
        // Callbacks aren't comparable, so two probes are treated as equal:
        // The reconciler then never tears the control down and re-subscribes on re-render.
        // The subscription (and the callback it captured) is established once, at mount.
        other.as_any().downcast_ref::<SizeProbe>().is_some()
    }

    fn clone_dyn(&self) -> Box<dyn CustomElement> {
        Box::new(SizeProbe {
            on_resize: self.on_resize.clone(),
        })
    }

    fn mount(&self, backend: &mut dyn Backend) -> ControlId {
        // report as a border
        let id = backend.create(ControlKind::Border);

        // Stretch to fill the grid cell so SizeChanged reports the cell size, which equals the child's rendered size.
        backend.set_prop(
            id,
            Prop::HorizontalAlignment,
            &PropValue::I32(HorizontalAlignment::Stretch.0),
        );
        backend.set_prop(
            id,
            Prop::VerticalAlignment,
            &PropValue::I32(VerticalAlignment::Stretch.0),
        );

        if let Some(on_resize) = self.on_resize.clone()
            && let Some(native) = backend.get_native_element(id)
            && let Ok(fe) = native.cast::<IFrameworkElement>()
        {
            let size_revoker = fe.SizeChanged(move |_sender, args| {
                if let Some(args) = args.as_ref()
                    && let Ok(s) = args.NewSize()
                {
                    on_resize(f64::from(s.width), f64::from(s.height));
                }
            });

            // Leak the revoker so the subscription lives for the control's lifetime.
            // This mirrors the framework's SwapChainPanel::on_resize).
            if let Ok(size_revoker) = size_revoker {
                #[expect(
                    clippy::mem_forget,
                    reason = "subscription must outlive this scope; mirrors framework"
                )]
                std::mem::forget(size_revoker);
            }
        }

        id
    }

    fn update(&self, _prev: &dyn CustomElement, _id: ControlId, _backend: &mut dyn Backend) {
        // Stateless control: the SizeChanged subscription persists for the control's lifetime, so there is nothing to diff.
    }
}
