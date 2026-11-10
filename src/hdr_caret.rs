use gpui_kit::{Bounds, Hsla, Pixels, Window};

#[cfg(target_os = "macos")]
mod mac {
    use std::cell::RefCell;
    use std::collections::HashMap;

    use gpui_kit::{Bounds, Hsla, Pixels, Rgba, Window};
    use objc2::rc::Retained;
    use objc2::runtime::ProtocolObject;
    use objc2_app_kit::NSView;
    use objc2_core_foundation::{CGPoint, CGRect, CGSize};
    use objc2_core_graphics::{CGColorSpace, kCGColorSpaceExtendedLinearSRGB};
    use objc2_metal::{
        MTLClearColor, MTLCommandBuffer, MTLCommandEncoder, MTLCommandQueue,
        MTLCreateSystemDefaultDevice, MTLDevice, MTLLoadAction, MTLPixelFormat,
        MTLRenderPassDescriptor, MTLStoreAction,
    };
    use objc2_quartz_core::{CAMetalDrawable, CAMetalLayer, CATransaction};
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};

    const MAX_BOOST: f64 = 3.;

    struct Glow {
        view: Retained<NSView>,
        layer: Retained<CAMetalLayer>,
        queue: Retained<ProtocolObject<dyn MTLCommandQueue>>,
        drawn: Option<([f64; 4], [u32; 4])>,
    }

    thread_local! {
        static GLOWS: RefCell<HashMap<usize, Glow>> = RefCell::new(HashMap::new());
    }

    fn view_of(window: &Window) -> Option<Retained<NSView>> {
        let handle = window.window_handle().ok()?;
        let RawWindowHandle::AppKit(handle) = handle.as_raw() else {
            return None;
        };
        // The handle points at the window's live content view; retaining it
        // keeps it valid for as long as the glow layer is attached.
        unsafe { Retained::retain(handle.ns_view.as_ptr().cast::<NSView>()) }
    }

    fn create(view: Retained<NSView>) -> Option<Glow> {
        let host = view.layer()?;
        let device = MTLCreateSystemDefaultDevice()?;
        let queue = device.newCommandQueue()?;
        let layer = CAMetalLayer::new();
        layer.setDevice(Some(&device));
        layer.setPixelFormat(MTLPixelFormat::RGBA16Float);
        layer.setWantsExtendedDynamicRangeContent(true);
        let space = CGColorSpace::with_name(Some(unsafe { kCGColorSpaceExtendedLinearSRGB }));
        layer.setColorspace(space.as_deref());
        layer.setOpaque(false);
        layer.setFramebufferOnly(true);
        layer.setHidden(true);
        layer.setZPosition(1000.);
        host.addSublayer(&layer);
        Some(Glow {
            view,
            layer,
            queue,
            drawn: None,
        })
    }

    fn linear(c: f32) -> f64 {
        let c = c as f64;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    }

    fn quiet(f: impl FnOnce()) {
        CATransaction::begin();
        CATransaction::setDisableActions(true);
        f();
        CATransaction::commit();
    }

    impl Glow {
        fn headroom(&self) -> f64 {
            self.view
                .window()
                .and_then(|w| w.screen())
                .map_or(1., |s| s.maximumExtendedDynamicRangeColorComponentValue())
        }

        fn scale(&self) -> f64 {
            self.view.window().map_or(1., |w| w.backingScaleFactor())
        }

        fn hide(&mut self) {
            if self.drawn.take().is_some() {
                quiet(|| self.layer.setHidden(true));
            }
        }

        fn show(&mut self, rect: Bounds<Pixels>, color: Hsla) -> bool {
            let boost = self.headroom().min(MAX_BOOST);
            if boost <= 1. {
                self.hide();
                return false;
            }
            let bounds = self.view.bounds();
            let (x, w, h) = (
                f64::from(f32::from(rect.origin.x)),
                f64::from(f32::from(rect.size.width)),
                f64::from(f32::from(rect.size.height)),
            );
            let top = f64::from(f32::from(rect.origin.y));
            let y = if self.view.isFlipped() {
                top
            } else {
                bounds.size.height - top - h
            };
            let rgba = Rgba::from(color);
            let clear = [
                linear(rgba.r) * boost,
                linear(rgba.g) * boost,
                linear(rgba.b) * boost,
                rgba.a as f64,
            ];
            let frame = [x, y, w, h];
            let key = clear.map(|v| (v * 1024.) as u32);
            if self.drawn == Some((frame, key)) {
                return true;
            }
            let scale = self.scale();
            quiet(|| {
                self.layer
                    .setFrame(CGRect::new(CGPoint::new(x, y), CGSize::new(w, h)));
                self.layer.setContentsScale(scale);
                self.layer
                    .setDrawableSize(CGSize::new((w * scale).ceil(), (h * scale).ceil()));
                self.layer.setHidden(false);
            });
            let Some(drawable) = self.layer.nextDrawable() else {
                return false;
            };
            let pass = MTLRenderPassDescriptor::new();
            let attachment = unsafe { pass.colorAttachments().objectAtIndexedSubscript(0) };
            attachment.setTexture(Some(&drawable.texture()));
            attachment.setLoadAction(MTLLoadAction::Clear);
            attachment.setStoreAction(MTLStoreAction::Store);
            attachment.setClearColor(MTLClearColor {
                red: clear[0],
                green: clear[1],
                blue: clear[2],
                alpha: clear[3],
            });
            let Some(buffer) = self.queue.commandBuffer() else {
                return false;
            };
            if let Some(encoder) = buffer.renderCommandEncoderWithDescriptor(&pass) {
                encoder.endEncoding();
            }
            buffer.presentDrawable(ProtocolObject::from_ref(&*drawable));
            buffer.commit();
            self.drawn = Some((frame, key));
            true
        }
    }

    pub fn show(window: &Window, rect: Bounds<Pixels>, color: Hsla) -> bool {
        let Some(view) = view_of(window) else {
            return false;
        };
        let id = Retained::as_ptr(&view) as usize;
        GLOWS.with(|glows| {
            let mut glows = glows.borrow_mut();
            if !glows.contains_key(&id) {
                match create(view) {
                    Some(glow) => glows.insert(id, glow),
                    None => return false,
                };
            }
            glows
                .get_mut(&id)
                .is_some_and(|glow| glow.show(rect, color))
        })
    }

    pub fn hide(window: &Window) {
        let Some(view) = view_of(window) else {
            return;
        };
        let id = Retained::as_ptr(&view) as usize;
        GLOWS.with(|glows| {
            if let Some(glow) = glows.borrow_mut().get_mut(&id) {
                glow.hide();
            }
        });
    }
}

pub fn supported() -> bool {
    cfg!(target_os = "macos")
}

#[cfg(target_os = "macos")]
pub fn show(window: &Window, rect: Bounds<Pixels>, color: Hsla) -> bool {
    mac::show(window, rect, color)
}

#[cfg(not(target_os = "macos"))]
pub fn show(_: &Window, _: Bounds<Pixels>, _: Hsla) -> bool {
    false
}

#[cfg(target_os = "macos")]
pub fn hide(window: &Window) {
    mac::hide(window)
}

#[cfg(not(target_os = "macos"))]
pub fn hide(_: &Window) {}
