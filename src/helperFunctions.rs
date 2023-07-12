use sysinfo::{CpuExt,Pid,  ProcessExt,System, SystemExt, PidExt, ProcessStatus, UserExt};
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid as nixPID;

pub fn print_process_tree(items:  &[Vec<String>], id: String,  depth: usize) {
    let padding = "  ".repeat(depth);
    let result = items.iter().find(|&v| v[0] == id);
    let mut p_name = "";
    if let Some(vec) = result {
        if let Some(name) = vec.get(11) {
            p_name= name;
        }
    }
    if id=="1" {

        println!("{}──({})",  p_name, id);
    } else{

        println!("{}└──── {} ({})", padding, p_name, id); 
    }
   
    let children: Vec<String> = items
        .iter()
        .filter(|&v| v[2] == id)  // Filter out sub-vectors with third element != "3"
        .map(|v| v[0].clone())  // Map each remaining sub-vector to its first element
        .collect();  // Collect the results into a new vector
    for child in children {
        print_process_tree(items, child, depth + 4);
    }

}


pub fn kill_process_and_children(pid: u32) {
    let  system = System::new();
    //let pid = Pid::from_raw(pid);
    for (_pid2, process) in system.processes() {
        if process.parent() == Some(Pid::from_u32(pid)) {
            kill_process_and_children(process.pid().as_u32());
        }
    }
     let pid = nixPID::from_raw(pid as i32);
    kill(pid, Signal::SIGTERM);


}