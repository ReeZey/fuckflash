mod utils;

use std::fs;
use std::os::raw::c_void;
use image::metadata::Orientation;
use image::ColorType;
use image::DynamicImage;
use image::ImageBuffer;
use image::RgbImage;
use image::GenericImage;
use image::Rgba;
use image::RgbaImage;
use retour::GenericDetour;
use winapi::shared::minwindef::{BOOL, DWORD, HINSTANCE, LPARAM, LPVOID, WPARAM};
use winapi::shared::ntdef::{HANDLE, LPCSTR};
use winapi::shared::windef::{HDC, HMENU, HWND};
use winapi::um::consoleapi::AllocConsole;
use winapi::um::wingdi::BITMAPINFO;
use winapi::um::winnt::DLL_PROCESS_ATTACH;
use winapi::um::winuser::{DestroyWindow, PostQuitMessage, SC_CLOSE, WM_CLOSE, WM_SYSCOMMAND, WS_CAPTION, WS_SYSMENU};

type CreateWindowExA = extern "system" fn(DWORD, LPCSTR, LPCSTR, DWORD, i32, i32, i32, i32, HWND, HMENU, HINSTANCE, LPVOID) -> HWND;
type SetWindowPos = extern "system" fn(HWND, HWND, i32, i32, i32, i32, u32) -> BOOL;
type WindowProc = extern "system" fn(HWND, u32, WPARAM, LPARAM) -> isize;
type SetDIBitsToDevice = extern "system" fn(HDC, i32, i32, DWORD, DWORD, i32, i32, u32, u32, *const u8, *const BITMAPINFO, u32) -> BOOL;

static mut CREATEWINDOW_DETOUR: Option<GenericDetour<CreateWindowExA>> = None;
static mut SETWINDOWPOS_DETOUR: Option<GenericDetour<SetWindowPos>> = None;
static mut WINDOWPROC_DETOUR: Option<GenericDetour<WindowProc>> = None;
static mut SETDIBITS_DETOUR: Option<GenericDetour<SetDIBitsToDevice>> = None;

unsafe fn setup_detour() {
    let create_window_ex_fn: CreateWindowExA = std::mem::transmute(
        utils::find_address("user32.dll", "CreateWindowExA")
    );
    let set_window_pos_fn: SetWindowPos = std::mem::transmute(
        utils::find_address("user32.dll", "SetWindowPos")
    );
    let window_proc_fn: WindowProc = std::mem::transmute(
        utils::find_address("user32.dll", "DefWindowProcA")
    );
    let set_dibits_fn: SetDIBitsToDevice = std::mem::transmute(
        utils::find_address("gdi32.dll", "SetDIBitsToDevice")
    );

    CREATEWINDOW_DETOUR = Some(utils::generate_detour(create_window_ex_fn, create_window_ex_hook));
    SETWINDOWPOS_DETOUR = Some(utils::generate_detour(set_window_pos_fn, set_window_pos_hook));
    WINDOWPROC_DETOUR = Some(utils::generate_detour(window_proc_fn, window_proc_hook));
    SETDIBITS_DETOUR = Some(utils::generate_detour(set_dibits_fn, set_dibits_hook));
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

        dw_style = WS_CAPTION | WS_SYSMENU;
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

        println!("windowify done");

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

        //HACK: find better way to get the window size.
        x = 0;
        y = 0;
        cx = 647;
        cy = 510;

        //println!("scale and stuff");

        return original.call(h_wnd, h_wnd_insert_after, x, y, cx, cy, u_flags);
    }
}

extern "system" fn window_proc_hook(
    h_wnd: HWND,
    u_msg: u32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> isize {
    unsafe {
        let original = WINDOWPROC_DETOUR.as_mut().unwrap();

        match u_msg {
            WM_SYSCOMMAND => {
                if w_param == SC_CLOSE as usize {
                    //HACK: find better way to close the window.
                    DestroyWindow(h_wnd);
                    return 0;
                }
            }
            _ => {}
        }

        return original.call(h_wnd, u_msg, w_param, l_param);
    }
}

static mut img_counter: i32 = 0;

extern "system" fn set_dibits_hook(
    hdc: HDC,
    x_dest: i32,
    y_dest: i32,
    width: u32,
    height: u32,
    x_src: i32,
    y_src: i32,
    start_scan: u32,
    scan_lines: u32,
    bits: *const u8,
    bmi: *const BITMAPINFO,
    color_use: u32,
) -> BOOL {
    unsafe {
        let original = SETDIBITS_DETOUR.as_mut().unwrap();

        if !bmi.is_null() {
            let header = bmi.as_ref().unwrap();
            let aaa = header.bmiHeader.biWidth as u32;
            let bbb = header.bmiHeader.biHeight as u32;

            let mut img = DynamicImage::new(aaa, bbb, ColorType::Rgb8);
            for y in 0..bbb {
                for x in 0..aaa {
                    let index = (y * aaa + x) as usize * 3;
                    //why are you reversed?
                    let b = *bits.offset(index as isize);
                    let g = *bits.offset(index as isize + 1);
                    let r = *bits.offset(index as isize + 2);
                    img.put_pixel(x, y, Rgba([r, g, b, 255]));
                }
            }
            img.apply_orientation(Orientation::FlipVertical);
            img.save(format!("debug/{}.png", img_counter)).unwrap();
            img_counter += 1;
        }

        return original.call(hdc, x_dest, y_dest, width, height, x_src, y_src, start_scan, scan_lines, bits, bmi, color_use);
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
