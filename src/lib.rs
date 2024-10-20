use std::ffi::CString;
use std::os::raw::c_void;

use retour::GenericDetour;
use winapi::shared::minwindef::BOOL;
use winapi::um::libloaderapi::{GetModuleHandleA, GetProcAddress};
use winapi::um::winnt::PCSTR;
use winapi::um::winuser::*;

type CreateWindowExA = extern "system" fn(u32, *const i8, *const i8, u32, i32, i32, i32, i32, i32, i32, i32, i32) -> i32;
type SetWindowPos = extern "system" fn(i32, i32, i32, i32, i32, i32, u32) -> BOOL;

static mut CREATEWINDOW_DETOUR: Option<GenericDetour<CreateWindowExA>> = None;
static mut SETWINDOWPOS_DETOUR: Option<GenericDetour<SetWindowPos>> = None;

static mut FIRST_CALL_CREATE: bool = true;

unsafe fn setup_detour() {
    let user32 = GetModuleHandleA(PCSTR::from(CString::new("user32.dll").unwrap().as_ptr()));
    assert!(user32.is_null() == false);
    
    let create_window_ex_a_address= GetProcAddress(
        user32,
        PCSTR::from(CString::new("CreateWindowExA").unwrap().as_ptr()),
    ) as *const c_void;
    assert!(create_window_ex_a_address.is_null() == false);

    let set_window_pos_address= GetProcAddress(
        user32,
        PCSTR::from(CString::new("SetWindowPos").unwrap().as_ptr()),
    ) as *const c_void;
    assert!(create_window_ex_a_address.is_null() == false);

    let create_window_ex_fn = std::mem::transmute::<*const c_void, CreateWindowExA>(create_window_ex_a_address);
    let detour = GenericDetour::new(create_window_ex_fn, create_window_ex_hook).unwrap();
    detour.enable().unwrap();
    CREATEWINDOW_DETOUR = Some(detour);

    let set_window_pos_fn = std::mem::transmute::<*const c_void, SetWindowPos>(set_window_pos_address);
    let detour = GenericDetour::new(set_window_pos_fn, set_window_pos_hook).unwrap();
    detour.enable().unwrap();
    SETWINDOWPOS_DETOUR = Some(detour);
}
extern "system" fn create_window_ex_hook(
    mut dw_ex_style: u32,
    lp_class_name: *const i8,
    lp_window_name: *const i8,
    mut dw_style: u32,
    x: i32,
    y: i32,
    n_width: i32,
    n_height: i32,
    h_wnd_parent: i32,
    h_menu: i32,
    h_instance: i32,
    lp_param: i32,
) -> i32 {

    unsafe {
        let original = CREATEWINDOW_DETOUR.as_mut().unwrap();

        if FIRST_CALL_CREATE {
            dw_style = WS_CAPTION | WS_SYSMENU;
            dw_ex_style = 0;

            FIRST_CALL_CREATE = false;
        }

        return original.call(dw_ex_style,
            lp_class_name,
            lp_window_name,
            dw_style,
            x,
            y,
            n_width,
            n_height,
            h_wnd_parent,
            h_menu,
            h_instance,
            lp_param
        );
    }
}

extern "system" fn set_window_pos_hook(
    h_wnd: i32,
    h_wnd_insert_after: i32,
    mut x: i32,
    mut y: i32,
    mut cx: i32,
    mut cy: i32,
    u_flags: u32,
) -> BOOL {
    unsafe {
        let original = SETWINDOWPOS_DETOUR.as_mut().unwrap();

        x = 0;
        y = 0;
        cx = 647;
        cy = 510;

        return original.call(h_wnd, h_wnd_insert_after, x, y, cx, cy, u_flags);
    }
}


#[no_mangle]
unsafe extern "system" fn DllMain(_h_instance: *mut c_void, reason: u32, _: *mut c_void) -> BOOL {
    match reason {
        1 => {
            println!("DLL Attached!");
            setup_detour();
        }
        0 => {
            println!("DLL Detached!");
        }
        _ => {}
    }

    return 1;
}
