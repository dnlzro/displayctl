use std::env;
use std::ffi::{CStr, CString, c_char, c_uchar, c_void};
use std::process;
use std::ptr;

// Opaque CoreGraphics types
enum CGDisplayConfig {}
type CGDisplayConfigRef = *mut CGDisplayConfig;

type CGDirectDisplayID = u32;
type CGDisplayCount = u32;
type CGError = i32;

enum CGDisplayMode {}

// Opaque CoreFoundation types
enum CFAllocator {}
type CFAllocatorRef = *const CFAllocator;

#[allow(clippy::upper_case_acronyms)]
enum CFString {}
type CFStringRef = *const CFString;

#[allow(clippy::upper_case_acronyms)]
enum CFUUID {}
type CFUUIDRef = *const CFUUID;

#[allow(clippy::upper_case_acronyms)]
enum CFDictionary {}
type CFDictionaryRef = *const CFDictionary;
type CFMutableDictionaryRef = *mut CFDictionary;

// IOKit types
#[allow(non_camel_case_types)]
type io_object_t = u32;
#[allow(non_camel_case_types)]
type io_iterator_t = u32;
#[allow(non_camel_case_types)]
type io_service_t = u32;
#[allow(non_camel_case_types)]
type kern_return_t = i32;

const K_CG_ERROR_SUCCESS: CGError = 0;
const K_CG_CONFIGURE_PERMANENTLY: i32 = 2;
const K_CF_STRING_ENCODING_UTF8: u32 = 0x0800_0100;
const KERN_SUCCESS: kern_return_t = 0;

#[allow(clippy::duplicated_attributes)]
#[link(name = "CoreGraphics", kind = "framework")]
#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn CGBeginDisplayConfiguration(config: *mut CGDisplayConfigRef) -> CGError;
    fn CGCompleteDisplayConfiguration(config: CGDisplayConfigRef, option: i32) -> CGError;
    fn CGSConfigureDisplayEnabled(
        config: CGDisplayConfigRef,
        display: CGDirectDisplayID,
        enabled: bool,
    ) -> CGError;
    fn CGGetOnlineDisplayList(
        maxDisplays: u32,
        displays: *mut CGDirectDisplayID,
        displayCount: *mut CGDisplayCount,
    ) -> CGError;
    fn CGDisplayIsActive(display: CGDirectDisplayID) -> bool;
    fn CGDisplayIsInMirrorSet(display: CGDirectDisplayID) -> bool;
    fn CGDisplayCreateUUIDFromDisplayID(display: CGDirectDisplayID) -> CFUUIDRef;
    fn CGDisplayGetDisplayIDFromUUID(uuid: CFUUIDRef) -> CGDirectDisplayID;
    fn CGDisplayIsBuiltin(display: CGDirectDisplayID) -> bool;
    fn CGMainDisplayID() -> CGDirectDisplayID;
    fn CGDisplayCopyDisplayMode(display: CGDirectDisplayID) -> *mut CGDisplayMode;
    fn CGDisplayModeGetWidth(mode: *mut CGDisplayMode) -> usize;
    fn CGDisplayModeGetHeight(mode: *mut CGDisplayMode) -> usize;
    fn CGDisplayModeGetPixelWidth(mode: *mut CGDisplayMode) -> usize;
    fn CGDisplayModeGetPixelHeight(mode: *mut CGDisplayMode) -> usize;
    fn CGDisplayModeGetRefreshRate(mode: *mut CGDisplayMode) -> f64;
    fn CGDisplayModeRelease(mode: *mut CGDisplayMode);
}

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFUUIDCreateString(alloc: CFAllocatorRef, uuid: CFUUIDRef) -> CFStringRef;
    fn CFStringGetCString(
        theString: CFStringRef,
        buffer: *mut c_char,
        bufferSize: usize,
        encoding: u32,
    ) -> c_uchar;
    fn CFStringCreateWithCString(
        alloc: CFAllocatorRef,
        cStr: *const c_char,
        encoding: u32,
    ) -> CFStringRef;
    fn CFUUIDCreateFromString(alloc: CFAllocatorRef, string: CFStringRef) -> CFUUIDRef;
    fn CFRelease(cf: *const c_void);
    fn CFDictionaryGetValue(theDict: CFDictionaryRef, key: CFStringRef) -> *const c_void;
    fn CFDictionaryGetCount(theDict: CFDictionaryRef) -> isize;
    fn CFDictionaryGetKeysAndValues(
        theDict: CFDictionaryRef,
        keys: *mut *const c_void,
        values: *mut *const c_void,
    );
}

#[link(name = "IOKit", kind = "framework")]
unsafe extern "C" {
    fn IOServiceGetMatchingServices(
        masterPort: u32,
        matching: CFMutableDictionaryRef,
        existing: *mut io_iterator_t,
    ) -> kern_return_t;
    fn IOServiceMatching(name: *const c_char) -> CFMutableDictionaryRef;
    fn IOIteratorNext(iterator: io_iterator_t) -> io_service_t;
    fn IODisplayCreateInfoDictionary(service: io_service_t, options: u32)
    -> CFMutableDictionaryRef;
    fn IOObjectRelease(object: io_object_t) -> kern_return_t;
}

fn cf_release<T>(ptr: *const T) {
    if !ptr.is_null() {
        unsafe { CFRelease(ptr as *const c_void) }
    }
}

fn cfstring_to_string(cf_string: CFStringRef) -> Option<String> {
    if cf_string.is_null() {
        return None;
    }
    unsafe {
        let mut buffer = [0 as c_char; 256];
        let ok = CFStringGetCString(
            cf_string,
            buffer.as_mut_ptr(),
            buffer.len(),
            K_CF_STRING_ENCODING_UTF8,
        );
        if ok == 0 {
            return None;
        }
        CStr::from_ptr(buffer.as_ptr())
            .to_str()
            .ok()
            .map(String::from)
    }
}

fn cfstring_from_str(s: &str) -> Option<CFStringRef> {
    let c_string = CString::new(s).ok()?;
    unsafe {
        let ptr =
            CFStringCreateWithCString(ptr::null(), c_string.as_ptr(), K_CF_STRING_ENCODING_UTF8);
        if ptr.is_null() { None } else { Some(ptr) }
    }
}

fn display_id_to_uuid(display_id: CGDirectDisplayID) -> Option<String> {
    unsafe {
        let cf_uuid = CGDisplayCreateUUIDFromDisplayID(display_id);
        if cf_uuid.is_null() {
            return None;
        }
        let cf_string = CFUUIDCreateString(ptr::null(), cf_uuid);
        cf_release(cf_uuid);
        if cf_string.is_null() {
            return None;
        }
        let result = cfstring_to_string(cf_string);
        cf_release(cf_string);
        result
    }
}

fn uuid_to_display_id(uuid_str: &str) -> Option<CGDirectDisplayID> {
    let c_string = CString::new(uuid_str).ok()?;
    unsafe {
        let cf_string =
            CFStringCreateWithCString(ptr::null(), c_string.as_ptr(), K_CF_STRING_ENCODING_UTF8);
        if cf_string.is_null() {
            return None;
        }
        let cf_uuid = CFUUIDCreateFromString(ptr::null(), cf_string);
        cf_release(cf_string);
        if cf_uuid.is_null() {
            return None;
        }
        let display_id = CGDisplayGetDisplayIDFromUUID(cf_uuid);
        cf_release(cf_uuid);
        if display_id == 0 {
            None
        } else {
            Some(display_id)
        }
    }
}

struct DisplayMode(*mut CGDisplayMode);

impl DisplayMode {
    fn copy(display_id: CGDirectDisplayID) -> Option<Self> {
        unsafe {
            let ptr = CGDisplayCopyDisplayMode(display_id);
            if ptr.is_null() { None } else { Some(Self(ptr)) }
        }
    }

    fn width(&self) -> usize {
        unsafe { CGDisplayModeGetWidth(self.0) }
    }

    fn height(&self) -> usize {
        unsafe { CGDisplayModeGetHeight(self.0) }
    }

    fn pixel_width(&self) -> usize {
        unsafe { CGDisplayModeGetPixelWidth(self.0) }
    }

    fn pixel_height(&self) -> usize {
        unsafe { CGDisplayModeGetPixelHeight(self.0) }
    }

    fn refresh_rate(&self) -> f64 {
        unsafe { CGDisplayModeGetRefreshRate(self.0) }
    }
}

impl Drop for DisplayMode {
    fn drop(&mut self) {
        unsafe { CGDisplayModeRelease(self.0) }
    }
}

fn get_display_name(display_id: CGDirectDisplayID) -> Option<String> {
    let uuid = display_id_to_uuid(display_id)?;

    unsafe {
        let c_name = CString::new("IODisplayConnect").ok()?;
        let matching = IOServiceMatching(c_name.as_ptr());
        if matching.is_null() {
            return None;
        }

        let mut iter: io_iterator_t = 0;
        if IOServiceGetMatchingServices(0, matching, &mut iter) != KERN_SUCCESS {
            return None;
        }

        let mut result = None;
        loop {
            let service = IOIteratorNext(iter);
            if service == 0 {
                break;
            }

            if let Some(name) = display_name_from_service(service, &uuid) {
                result = Some(name);
                IOObjectRelease(service);
                break;
            }
            IOObjectRelease(service);
        }
        IOObjectRelease(iter);
        result
    }
}

fn display_name_from_service(service: io_service_t, target_uuid: &str) -> Option<String> {
    unsafe {
        let info = IODisplayCreateInfoDictionary(service, 1);
        if info.is_null() {
            return None;
        }

        let uuid_key = cfstring_from_str("IODisplayUUID")?;
        let display_uuid = CFDictionaryGetValue(info as CFDictionaryRef, uuid_key);
        cf_release(uuid_key);

        let matches = !display_uuid.is_null()
            && cfstring_to_string(display_uuid as CFStringRef).as_deref() == Some(target_uuid);

        if !matches {
            cf_release(info as *const CFDictionary);
            return None;
        }

        let name_key = cfstring_from_str("DisplayProductName")?;
        let names = CFDictionaryGetValue(info as CFDictionaryRef, name_key);
        cf_release(name_key);
        cf_release(info as *const CFDictionary);

        if names.is_null() {
            return None;
        }

        let count = CFDictionaryGetCount(names as CFDictionaryRef) as usize;
        if count == 0 {
            return None;
        }

        let mut values: [CFStringRef; 4] = [ptr::null(); 4];
        let n = count.min(values.len());
        CFDictionaryGetKeysAndValues(
            names as CFDictionaryRef,
            ptr::null_mut(),
            values.as_mut_ptr() as *mut *const c_void,
        );

        values[..n]
            .iter()
            .copied()
            .find(|&v| !v.is_null())
            .and_then(cfstring_to_string)
    }
}

fn get_online_displays() -> Vec<CGDirectDisplayID> {
    unsafe {
        let mut count: CGDisplayCount = 0;
        if CGGetOnlineDisplayList(u32::MAX, ptr::null_mut(), &mut count) != K_CG_ERROR_SUCCESS {
            return Vec::new();
        }

        let mut displays = vec![0; count as usize];
        if CGGetOnlineDisplayList(u32::MAX, displays.as_mut_ptr(), &mut count) != K_CG_ERROR_SUCCESS
        {
            return Vec::new();
        }

        displays.truncate(count as usize);
        displays
    }
}

fn display_status(display_id: CGDirectDisplayID) -> &'static str {
    unsafe {
        if CGDisplayIsActive(display_id) {
            "active"
        } else if CGDisplayIsInMirrorSet(display_id) {
            "mirror"
        } else {
            "inactive"
        }
    }
}

fn display_mode_info(display_id: CGDirectDisplayID) -> (String, String, String) {
    let mode = DisplayMode::copy(display_id);
    let Some(mode) = mode else {
        return (String::from("-"), String::from("-"), String::from("-"));
    };

    let w = mode.width();
    let h = mode.height();
    let pw = mode.pixel_width();
    let ph = mode.pixel_height();
    let rate = mode.refresh_rate();

    let mode_str = format!("{}x{}", w, h);
    let native = if w != pw || h != ph {
        format!("{}x{}", pw, ph)
    } else {
        String::from("-")
    };
    let refresh = if rate > 0.0 {
        format!("{:.2}Hz", rate)
    } else {
        String::from("-")
    };

    (mode_str, native, refresh)
}

fn list_displays() {
    let displays = get_online_displays();
    if displays.is_empty() {
        return;
    }

    let mut rows: Vec<(String, &'static str, String, String, String, String)> = Vec::new();
    for &display in &displays {
        let uuid = display_id_to_uuid(display).unwrap_or_else(|| String::from("unknown"));
        let status = display_status(display);
        let builtin = unsafe { CGDisplayIsBuiltin(display) };

        let name = get_display_name(display).unwrap_or_else(|| {
            if builtin {
                String::from("Built-in Display")
            } else {
                String::from("External Display")
            }
        });

        let (mode, native, refresh) = display_mode_info(display);
        rows.push((name, status, mode, native, refresh, uuid));
    }

    let w0 = rows.iter().map(|r| r.0.len()).max().unwrap_or(4).max(4);
    let w1 = rows.iter().map(|r| r.1.len()).max().unwrap_or(6).max(6);
    let w2 = rows.iter().map(|r| r.2.len()).max().unwrap_or(4).max(4);
    let w3 = rows.iter().map(|r| r.3.len()).max().unwrap_or(6).max(6);
    let w4 = rows.iter().map(|r| r.4.len()).max().unwrap_or(7).max(7);
    let w5 = rows.iter().map(|r| r.5.len()).max().unwrap_or(4).max(4);

    println!(
        "{:<w0$}  {:<w1$}  {:<w2$}  {:<w3$}  {:<w4$}  {:<w5$}",
        "NAME", "STATUS", "MODE", "NATIVE", "REFRESH", "UUID"
    );

    for (name, status, mode, native, refresh, uuid) in &rows {
        println!(
            "{:<w0$}  {:<w1$}  {:<w2$}  {:<w3$}  {:<w4$}  {:<w5$}",
            name, status, mode, native, refresh, uuid
        );
    }
}

fn set_enabled(uuid_str: &str, enabled: bool) -> Result<(), String> {
    let display_id = uuid_to_display_id(uuid_str)
        .ok_or_else(|| format!("Invalid UUID or display not found: {}", uuid_str))?;

    let displays = get_online_displays();
    if !displays.contains(&display_id) {
        return Err(format!("Display {} is not online", uuid_str));
    }

    unsafe {
        let currently_enabled = CGDisplayIsActive(display_id) || CGDisplayIsInMirrorSet(display_id);
        if currently_enabled == enabled {
            return Ok(());
        }

        let mut config: CGDisplayConfigRef = ptr::null_mut();
        let err = CGBeginDisplayConfiguration(&mut config);
        if err != K_CG_ERROR_SUCCESS {
            return Err(format!(
                "Failed to begin display configuration (error: {})",
                err
            ));
        }

        let err = CGSConfigureDisplayEnabled(config, display_id, enabled);
        if err != K_CG_ERROR_SUCCESS {
            return Err(format!(
                "Failed to set display enabled={} (error: {})",
                enabled, err
            ));
        }

        let err = CGCompleteDisplayConfiguration(config, K_CG_CONFIGURE_PERMANENTLY);
        if err != K_CG_ERROR_SUCCESS {
            return Err(format!(
                "Failed to complete display configuration (error: {})",
                err
            ));
        }
    }

    Ok(())
}

fn print_usage(program: &str) {
    println!("{}", env!("CARGO_PKG_DESCRIPTION"));
    println!();
    println!("Usage: {} <COMMAND>", program);
    println!();
    println!("Commands:");
    println!("  list            List online displays");
    println!("  disable <UUID>  Disable a display by UUID");
    println!("  enable <UUID>   Enable a display by UUID");
    println!();
    println!("Options:");
    println!("  -h, --help     Print help");
    println!("  -V, --version  Print version");
}

fn print_version() {
    println!("{}", env!("CARGO_PKG_VERSION"));
}

fn main() {
    unsafe {
        CGMainDisplayID();
    }

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage(&args[0]);
        process::exit(1);
    }

    match args[1].as_str() {
        "help" | "--help" | "-h" => {
            print_usage(&args[0]);
            process::exit(0);
        }
        "--version" | "-V" => {
            print_version();
            process::exit(0);
        }
        "list" => list_displays(),
        "disable" | "enable" => {
            if args.len() != 3 {
                print_usage(&args[0]);
                process::exit(1);
            }
            let enabled = args[1] == "enable";
            if let Err(e) = set_enabled(&args[2], enabled) {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
        _ => {
            print_usage(&args[0]);
            process::exit(1);
        }
    }
}
