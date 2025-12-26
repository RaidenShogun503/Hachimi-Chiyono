use egui::{Event, Pos2, RawInput, TouchDeviceId, TouchId, TouchPhase};
use objc::runtime::{Class, Imp, Method, Object, Sel};
use objc::{msg_send, sel, sel_impl};
use std::sync::{Arc, Mutex, Once};

static mut INPUT_STATE: Option<Arc<Mutex<RawInput>>> = None;
static INIT: Once = Once::new();

pub fn init() {
    unsafe {
        INIT.call_once(|| {
            INPUT_STATE = Some(Arc::new(Mutex::new(RawInput::default())));
            hook_input();
        });
    }
}

pub fn get_input() -> Option<RawInput> {
    unsafe {
        if let Some(state) = &INPUT_STATE {
            let mut guard = state.lock().unwrap();
            let input = guard.clone();
            // Clear transient events but keep state?
            // RawInput: events are transient. modifiers/pointers usually persistent.
            // egui::RawInput::take() effectively moves events out.
            // But we need to keep `screen_rect` etc.

            // Actually, we should accumulate events and take them.
            // Let's just return what we have and clear events.
            guard.events.clear();

            Some(input)
        } else {
            None
        }
    }
}

// Hook logic similar to gui_impl swizzling
unsafe fn hook_input() {
    let app_class = Class::get("UIApplication");
    if let Some(cls) = app_class {
        let sel_send = sel!(sendEvent:);
        extern "C" {
            fn class_getInstanceMethod(cls: *const Class, sel: Sel) -> *mut Method;
        }
        let method_ptr = class_getInstanceMethod(cls as *const Class, sel_send);
        if !method_ptr.is_null() {
            let new_imp: Imp = std::mem::transmute(
                send_event_hook as extern "C" fn(*mut Object, Sel, *mut Object),
            );
            let old_imp = objc::runtime::method_setImplementation(&mut *method_ptr, new_imp);
            ORIG_SEND_EVENT = std::mem::transmute(old_imp);
            log::info!("Swizzled UIApplication sendEvent:");
        }
    }
}

static mut ORIG_SEND_EVENT: extern "C" fn(*mut Object, Sel, *mut Object) = dummy_send_event;

extern "C" fn dummy_send_event(_: *mut Object, _: Sel, _: *mut Object) {}

extern "C" fn send_event_hook(this: *mut Object, cmd: Sel, event: *mut Object) {
    unsafe {
        // Process event
        process_event(event);

        // Call original
        ORIG_SEND_EVENT(this, cmd, event);
    }
}

unsafe fn process_event(event: *mut Object) {
    // Check if type is UIEventTypeTouches (0)
    let type_: isize = msg_send![event, type];
    if type_ == 0 {
        let touches: *mut Object = msg_send![event, allTouches];
        // touches is NSSet
        let count: usize = msg_send![touches, count];
        if count > 0 {
            let enumerator: *mut Object = msg_send![touches, objectEnumerator];
            loop {
                let touch: *mut Object = msg_send![enumerator, nextObject];
                if touch.is_null() {
                    break;
                }

                let phase: isize = msg_send![touch, phase];
                let touch_view: *mut Object = msg_send![touch, view];
                let location: CGPoint = msg_send![touch, locationInView: touch_view];

                // Map phase
                let phase = match phase {
                    0 => TouchPhase::Start,
                    1 => TouchPhase::Move,
                    3 => TouchPhase::End,
                    4 => TouchPhase::Cancel,
                    _ => TouchPhase::Move, // Stationary?
                };

                // Update state
                if let Some(state) = &INPUT_STATE {
                    if let Ok(mut guard) = state.lock() {
                        // pointer id?
                        let touch_id = touch as u64; // Use pointer as ID

                        guard.events.push(Event::Touch {
                            device_id: TouchDeviceId(0),
                            id: TouchId::from(touch_id),
                            phase,
                            pos: Pos2::new(location.x as f32, location.y as f32),
                            force: None,
                        });
                    }
                }
            }
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct CGPoint {
    x: f64,
    y: f64,
}
