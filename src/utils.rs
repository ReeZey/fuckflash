use std::ffi::{c_void, CString};

use retour::GenericDetour;
use winapi::um::libloaderapi::{GetModuleHandleA, GetProcAddress};

pub fn find_address(module: &str, function: &str) -> *const c_void {
    let module = unsafe { GetModuleHandleA(CString::new(module).unwrap().as_ptr()) };
    assert!(module.is_null() == false);

    unsafe { GetProcAddress(module, CString::new(function).unwrap().as_ptr()) as *const c_void }
}

pub unsafe fn generate_detour<T>(function: T, hook: T) -> GenericDetour<T> 
where T: retour::Function
{
    let detour = GenericDetour::new(function, hook).unwrap();
    detour.enable().unwrap();
    detour
}