use std::{env, path::PathBuf, process::Command};
use dll_syringe::{process::OwnedProcess, Syringe};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("Usage: loader.exe <path to executable>");
        return;
    }

    let exe_path = PathBuf::from(env::current_exe().unwrap());
    let dir_path = exe_path.parent().unwrap();
    let dll_path = dir_path.join("fuckflash.dll");
    
    let child = Command::new(&args[1]).spawn().unwrap();
    
    let pid = child.id();
    println!("started process {}", pid);
    let target_process = OwnedProcess::from_pid(pid).unwrap();

    let syringe = Syringe::for_process(target_process);

    let injected_payload = syringe.inject(&dll_path).unwrap();
    println!("Injected {:?}", injected_payload);

    let output = child.wait_with_output();
    println!("{:?}", output);

    //debug
    //std::io::stdin().read_line(&mut String::new()).unwrap();
}
