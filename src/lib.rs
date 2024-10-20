use std::ffi::CString;
use std::os::raw::c_void;

use retour::GenericDetour;
use winapi::shared::minwindef::BOOL;
use winapi::um::libloaderapi::{GetModuleHandleA, GetProcAddress};
use winapi::um::winnt::PCSTR;

type CreateWindowExA = extern "system" fn(u32, *const i8, *const i8, u32, i32, i32, i32, i32, i32, i32, i32, i32) -> i32;

static mut CREATEWINDOW_DETOUR: Option<GenericDetour<CreateWindowExA>> = None;

unsafe fn setup_detour() {
    let user32 = GetModuleHandleA(PCSTR::from(CString::new("user32.dll").unwrap().as_ptr()));
    assert!(user32.is_null() == false);
    
    let create_window_ex_a_address= GetProcAddress(
        user32,
        PCSTR::from(CString::new("CreateWindowExA").unwrap().as_ptr()),
    ) as *const c_void;
    assert!(create_window_ex_a_address.is_null() == false);

    let create_window_ex = std::mem::transmute::<*const c_void, CreateWindowExA>(create_window_ex_a_address);
    let detour = GenericDetour::new(create_window_ex, create_window_ex_hook).unwrap();
    detour.enable().unwrap();
    CREATEWINDOW_DETOUR = Some(detour);
}
extern "system" fn create_window_ex_hook(
    mut dw_ex_style: u32,
    lp_class_name: *const i8,
    lp_window_name: *const i8,
    mut dwStyle: u32,
    mut x: i32,
    mut y: i32,
    mut n_width: i32,
    mut n_height: i32,
    h_wnd_parent: i32,
    h_menu: i32,
    h_instance: i32,
    lp_param: i32,
) -> i32 {

    unsafe {
        let original = CREATEWINDOW_DETOUR.as_mut().unwrap();

        if dwStyle == 2181038080 {
            dwStyle = 0x00CF0000;
            dw_ex_style = 0;

            n_width = 800;
            n_height = 600;

            x = 0;
            y = 0;
        }

        return original.call(dw_ex_style,
            lp_class_name,
            lp_window_name,
            dwStyle,
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
