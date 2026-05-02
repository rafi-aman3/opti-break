// macOS Spaces helper.
//
// When a fullscreen app is in the foreground, it lives in its own Space and
// our overlay window does not render there (we use a plain NSWindow, not an
// NSPanel, after the NSPanel class-swap caused foreign-exception aborts).
//
// Workaround: at break start, detect any managed display whose current Space
// is a fullscreen Space and switch that display to its first user (desktop)
// Space, where the overlay is visible. Implemented via private CoreGraphics
// "Skylight" APIs (`CGSManagedDisplaySetCurrentSpace` et al.) — undocumented
// but stable since ~10.10 and widely used by window-management apps.

#![cfg(target_os = "macos")]

use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2_foundation::{NSArray, NSDictionary, NSNumber, NSString};

type CGSConnectionID = std::os::raw::c_int;
type CGSSpaceID = u64;

/// Skylight Space-type tag for fullscreen-app Spaces.
/// User/desktop Spaces use type 0; system Spaces use type 2.
const SPACE_TYPE_FULLSCREEN: i64 = 4;

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn _CGSDefaultConnection() -> CGSConnectionID;
    fn CGSCopyManagedDisplaySpaces(cid: CGSConnectionID) -> *mut AnyObject;
    fn CGSManagedDisplaySetCurrentSpace(
        cid: CGSConnectionID,
        display: *const AnyObject,
        space: CGSSpaceID,
    );
}

/// If any managed display currently shows a fullscreen Space, switch it to
/// that display's first non-fullscreen Space. No-op if every display is
/// already on a regular desktop Space.
pub fn leave_fullscreen_spaces() {
    unsafe {
        let cid = _CGSDefaultConnection();
        let raw = CGSCopyManagedDisplaySpaces(cid);
        if raw.is_null() {
            return;
        }

        // CFArray is toll-free bridged to NSArray. CGSCopy* returns a +1
        // retained pointer, so wrap with Retained::from_raw to take ownership.
        let displays_ptr: *mut NSArray<NSDictionary<NSString, AnyObject>> = raw.cast();
        let Some(displays) = Retained::from_raw(displays_ptr) else {
            return;
        };

        for display in displays.iter() {
            let Some(display_id) = dict_string(&display, "Display Identifier") else {
                continue;
            };
            let Some(spaces) = dict_array(&display, "Spaces") else {
                continue;
            };
            let Some(current) = dict_dict(&display, "Current Space") else {
                continue;
            };

            if dict_i64(&current, "type") != Some(SPACE_TYPE_FULLSCREEN) {
                continue;
            }

            let Some(target) = first_non_fullscreen_space(&spaces) else {
                continue;
            };

            CGSManagedDisplaySetCurrentSpace(
                cid,
                Retained::as_ptr(&display_id) as *const AnyObject,
                target,
            );
        }
    }
}

fn first_non_fullscreen_space(
    spaces: &NSArray<NSDictionary<NSString, AnyObject>>,
) -> Option<CGSSpaceID> {
    for space in spaces.iter() {
        if dict_i64(&space, "type") != Some(SPACE_TYPE_FULLSCREEN) {
            if let Some(id) = dict_i64(&space, "ManagedSpaceID") {
                return Some(id as CGSSpaceID);
            }
        }
    }
    None
}

// ── NSDictionary accessors ────────────────────────────────────────────────────

fn ns_key(s: &str) -> Retained<NSString> {
    NSString::from_str(s)
}

fn dict_string(
    dict: &NSDictionary<NSString, AnyObject>,
    key: &str,
) -> Option<Retained<NSString>> {
    let val = dict.objectForKey(&ns_key(key))?;
    let ptr: *const AnyObject = Retained::as_ptr(&val).cast();
    let s: *const NSString = ptr.cast();
    unsafe { Retained::retain(s.cast_mut()) }
}

fn dict_array(
    dict: &NSDictionary<NSString, AnyObject>,
    key: &str,
) -> Option<Retained<NSArray<NSDictionary<NSString, AnyObject>>>> {
    let val = dict.objectForKey(&ns_key(key))?;
    let ptr: *const AnyObject = Retained::as_ptr(&val).cast();
    let a: *const NSArray<NSDictionary<NSString, AnyObject>> = ptr.cast();
    unsafe { Retained::retain(a.cast_mut()) }
}

fn dict_dict(
    dict: &NSDictionary<NSString, AnyObject>,
    key: &str,
) -> Option<Retained<NSDictionary<NSString, AnyObject>>> {
    let val = dict.objectForKey(&ns_key(key))?;
    let ptr: *const AnyObject = Retained::as_ptr(&val).cast();
    let d: *const NSDictionary<NSString, AnyObject> = ptr.cast();
    unsafe { Retained::retain(d.cast_mut()) }
}

fn dict_i64(dict: &NSDictionary<NSString, AnyObject>, key: &str) -> Option<i64> {
    let val = dict.objectForKey(&ns_key(key))?;
    let ptr: *const AnyObject = Retained::as_ptr(&val).cast();
    let n: *const NSNumber = ptr.cast();
    let n = unsafe { Retained::retain(n.cast_mut())? };
    Some(n.longLongValue())
}
