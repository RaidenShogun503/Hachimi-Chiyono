use crate::ios::{input, renderer::MetalRenderer};
use metal::{Device, MetalDrawable};
use objc::runtime::{Class, Imp, Method, Object, Sel};
use objc::{msg_send, sel, sel_impl};
use std::ffi::c_void;
use std::sync::Once;

static INIT: Once = Once::new();
static mut ORIG_NEXT_DRAWABLE: usize = 0;
static mut RENDERER: Option<MetalRenderer> = None;
static mut EGUI_CTX: Option<egui::Context> = None;
static mut SHOW_MENU: bool = false;

pub fn init() {
    unsafe {
        INIT.call_once(|| {
            input::init();
            hook_metal();
        });
    }
}

pub fn draw() {
    // This function is kept for compatibility if called elsewhere, but we drive loop in hook
}

unsafe fn hook_metal() {
    let layer_class = Class::get("CAMetalLayer");
    if layer_class.is_none() {
        log::error!("CAMetalLayer class not found!");
        return;
    }
    let layer_class = layer_class.unwrap();

    let sel_next = sel!(nextDrawable);
    extern "C" {
        fn class_getInstanceMethod(cls: *const Class, sel: Sel) -> *mut Method;
    }

    let method_ptr = class_getInstanceMethod(layer_class as *const Class, sel_next);

    if !method_ptr.is_null() {
        let old_imp = (*method_ptr).implementation();
        log::info!("Found nextDrawable at {:?}", old_imp);

        let new_imp: Imp = std::mem::transmute(
            next_drawable_hook as extern "C" fn(*mut Object, Sel) -> *mut Object,
        );

        let prev_imp = objc::runtime::method_setImplementation(&mut *method_ptr, new_imp);

        let prev_imp_addr = prev_imp as usize;
        if prev_imp_addr != 0 {
            ORIG_NEXT_DRAWABLE = prev_imp_addr;
            log::info!("Swizzled CAMetalLayer nextDrawable (method_setImplementation)");
        } else {
            log::warn!("method_setImplementation returned NULL");
        }
    } else {
        log::error!("nextDrawable method not found on CAMetalLayer");
    }
}

extern "C" fn next_drawable_hook(this: *mut Object, cmd: Sel) -> *mut Object {
    unsafe {
        let func: extern "C" fn(*mut Object, Sel) -> *mut Object =
            std::mem::transmute(ORIG_NEXT_DRAWABLE);
        let drawable_obj = func(this, cmd);

        if !drawable_obj.is_null() {
            ensure_present_hook(drawable_obj);
        }

        drawable_obj
    }
}

static mut ORIG_PRESENT: usize = 0;
static HOOK_PRESENT_ONCE: Once = Once::new();

unsafe fn ensure_present_hook(drawable: *mut Object) {
    HOOK_PRESENT_ONCE.call_once(|| {
        let cls: *const Class = msg_send![drawable, class];
        let sel_present = sel!(present);

        extern "C" {
            fn class_getInstanceMethod(cls: *const Class, sel: Sel) -> *mut Method;
        }

        let method_ptr = class_getInstanceMethod(cls, sel_present);
        if !method_ptr.is_null() {
            let old_imp = (*method_ptr).implementation();
            log::info!("Found present at {:?}", old_imp);

            let new_imp: Imp = std::mem::transmute(present_hook as extern "C" fn(*mut Object, Sel));

            let prev_imp = objc::runtime::method_setImplementation(&mut *method_ptr, new_imp);
            ORIG_PRESENT = prev_imp as usize;
            log::info!("Swizzled CAMetalDrawable present");
        } else {
            log::error!("present method not found on drawable class");
        }
    });
}

extern "C" fn present_hook(this: *mut Object, cmd: Sel) {
    unsafe {
        // Render BEFORE present
        render_gui(this);

        let func: extern "C" fn(*mut Object, Sel) = std::mem::transmute(ORIG_PRESENT);
        func(this, cmd);
    }
}

unsafe fn render_gui(drawable_obj: *mut Object) {
    // Initialize Context and Renderer lazily
    if EGUI_CTX.is_none() {
        EGUI_CTX = Some(egui::Context::default());
        // Configure fonts/style here if needed
    }

    if RENDERER.is_none() {
        // Get device from drawable.layer.device
        use metal::foreign_types::ForeignType;

        let layer: *mut Object = msg_send![drawable_obj, layer];
        if !layer.is_null() {
            let device_obj: *mut Object = msg_send![layer, device];
            if !device_obj.is_null() {
                let device = Device::from_ptr(std::mem::transmute(device_obj));
                RENDERER = Some(MetalRenderer::new(device));
                log::info!("Initialized MetalRenderer");
            }
        }
    }

    // Render Loop
    if let (Some(ctx), Some(renderer)) = (&mut EGUI_CTX, &mut RENDERER) {
        let input = input::get_input().unwrap_or_default();

        let drawable_ref = &*(drawable_obj as *mut c_void as *const metal::MetalDrawableRef);
        let texture = drawable_ref.texture();
        let screen_size = (texture.width() as f32, texture.height() as f32);

        // Try to get scale from layer
        let layer: *mut Object = msg_send![drawable_obj, layer];
        let scale: f64 = if !layer.is_null() {
            msg_send![layer, contentsScale]
        } else {
            1.0
        };
        let pixels_per_point = scale as f32;

        ctx.set_pixels_per_point(pixels_per_point);

        let raw_input = input;

        let full_output = ctx.run(raw_input, |ctx| {
            if unsafe { SHOW_MENU } {
                let mut open = true;
                egui::Window::new("Hachimi Edge Setup")
                    .open(&mut open)
                    .show(ctx, |ui| {
                        ui.label("Successfully injected on iOS!");
                        if ui.button("Setup").clicked() {
                            log::info!("Setup clicked setup");
                        }

                        if ui.button("Unlock FPS (240)").clicked() {
                            log::info!("Unlocking FPS to 240");
                            unsafe {
                                // 1. Update Hachimi config so the hook respects it later
                                use crate::core::Hachimi;
                                if Hachimi::is_initialized() {
                                    Hachimi::instance()
                                        .target_fps
                                        .store(240, std::sync::atomic::Ordering::Relaxed);
                                }

                                // 2. Call Unity API immediately
                                // We need to resolve it manually because we can't easily import the hook fn from here without circular deps maybe?
                                // Or just use il2cpp_resolve_icall.
                                let func_addr = crate::il2cpp::api::il2cpp_resolve_icall(
                                    c"UnityEngine.Application::set_targetFrameRate(System.Int32)"
                                        .as_ptr(),
                                );
                                if func_addr != 0 {
                                    let func: extern "C" fn(i32) = std::mem::transmute(func_addr);
                                    func(240);
                                    log::info!(
                                        "Called UnityEngine.Application::set_targetFrameRate(240)"
                                    );
                                } else {
                                    log::error!("Failed to resolve set_targetFrameRate");
                                }
                            }
                        }
                    });
                if !open {
                    unsafe {
                        SHOW_MENU = false;
                    }
                }
            } else {
                egui::Window::new("FloatButton")
                    .title_bar(false)
                    .resizable(false)
                    .collapsible(false)
                    .auto_sized()
                    .default_pos([50.0, 100.0])
                    .show(ctx, |ui| {
                        let btn = egui::Button::new(egui::RichText::new("MENU").size(16.0));
                        if ui.add(btn).clicked() {
                            unsafe {
                                SHOW_MENU = true;
                            }
                        }
                    });
            }
        });

        let clipped_primitives = ctx.tessellate(full_output.shapes, full_output.pixels_per_point);

        for (id, delta) in full_output.textures_delta.set {
            renderer.update_texture(id, &delta);
        }

        renderer.render(texture, clipped_primitives, screen_size, pixels_per_point);
    }
}
