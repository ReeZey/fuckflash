mod utils;

use std::os::raw::c_void;

use retour::GenericDetour;
use winapi::shared::minwindef::{BOOL, DWORD, HINSTANCE, LPVOID};
use winapi::shared::ntdef::LPCSTR;
use winapi::shared::windef::{HMENU, HWND};
use winapi::um::consoleapi::AllocConsole;
use winapi::um::winnt::DLL_PROCESS_ATTACH;
use winapi::um::winuser::WS_CAPTION;

type CreateWindowExA = extern "system" fn(DWORD, LPCSTR, LPCSTR, DWORD, i32, i32, i32, i32, HWND, HMENU, HINSTANCE, LPVOID) -> HWND;
type SetWindowPos = extern "system" fn(HWND, HWND, i32, i32, i32, i32, u32) -> BOOL;

static mut CREATEWINDOW_DETOUR: Option<GenericDetour<CreateWindowExA>> = None;
static mut SETWINDOWPOS_DETOUR: Option<GenericDetour<SetWindowPos>> = None;

unsafe fn setup_detour() {
    let create_window_ex_fn: CreateWindowExA = std::mem::transmute(
        utils::find_address("user32.dll", "CreateWindowExA")
    );
    let set_window_pos_fn: SetWindowPos = std::mem::transmute(
        utils::find_address("user32.dll", "SetWindowPos")
    );

    CREATEWINDOW_DETOUR = Some(utils::generate_detour(create_window_ex_fn, create_window_ex_hook));
    SETWINDOWPOS_DETOUR = Some(utils::generate_detour(set_window_pos_fn, set_window_pos_hook));
}

extern "system" fn create_window_ex_hook(
    mut dw_ex_style: DWORD,
    lp_class_name: LPCSTR,
    lp_window_name: LPCSTR,
    mut dw_style: DWORD,
    x: i32,
    y: i32,
    n_width: i32,
    n_height: i32,
    h_wnd_parent: HWND,
    h_menu: HMENU,
    h_instance: HINSTANCE,
    lp_param: LPVOID,
) -> HWND {

    unsafe {
        let original = CREATEWINDOW_DETOUR.as_mut().unwrap();

        dw_style = WS_CAPTION;
        dw_ex_style = 0;

        let result= original.call(dw_ex_style,
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

        //TODO: verify that we are sure the first window is what window we want.
        //HACK: seems to work to just change first window.
        original.disable().unwrap();

        println!("hook done");

        result
    }
}

extern "system" fn set_window_pos_hook(
    h_wnd: HWND,
    h_wnd_insert_after: HWND,
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

        println!("scale and stuff");

        return original.call(h_wnd, h_wnd_insert_after, x, y, cx, cy, u_flags);
    }
}


#[no_mangle]
unsafe extern "system" fn DllMain(_h_instance: *mut c_void, reason: u32, _: *mut c_void) -> BOOL {
    match reason {
        DLL_PROCESS_ATTACH => {
            setup_detour();

            AllocConsole();
            println!("DLL LOADED!");
        }
        _ => {}
    }

    return 1;
}
