// use sysinfo::{CpuExt,Pid,  ProcessExt,System, SystemExt, PidExt, ProcessStatus, UserExt};
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid as nixPID;
use sysinfo::{Pid,  ProcessExt,System, SystemExt, PidExt};
use crossterm::event::{KeyModifiers, KeyCode};
use crate::App;



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



pub fn handle_key(key: crossterm::event::KeyEvent, app: &mut App) -> bool {

    let mut terminate = false;
    let mut selected_index: Option<usize> = None;

    match key.code {
                    
        KeyCode::Char(x) => if key.modifiers == KeyModifiers::SHIFT{

            match x {

                'P' => {
                    app.sort_by_what = 0;
                    
                },
                'U' => {
                    app.sort_by_what = 1;

                },
                'N' => {
                    app.sort_by_what = 3;

                },
                'C' => { 
                    app.sort_by_what = 8;

                },
                'M' => {
                    app.sort_by_what = 9;

                },
                'T' => {
                    app.sort_by_what = 11;

                },
                _ => {}
            }

        } else if key.modifiers == KeyModifiers::CONTROL{
            let mut start_index = 0;
            if let Some(index) = selected_index {
            // If there is a recorded index, start the search from the next process after the last selected process
            start_index = index + 1;
            }   
            // Search for the next process starting with 'a'
            if let Some(index) = app.items.iter().skip(start_index).position(|row| row[11].starts_with(x)) {
                // Do something with the matching process

                // Set the selected index to the index of the matching process
                app.state.select(Some(start_index + index));
                // Record the index of the last selected process
                selected_index = Some(start_index + index);
            } else {
                // If there are no more matches, start the search again from the beginning of the process list
                if let Some(index) = app.items.iter().position(|row| row[11].starts_with(x)) {
                    // Do something with the matching process

                    // Set the selected index to the index of the matching process
                    app.state.select(Some(index));
                    // Record the index of the last selected process
                    selected_index = Some(index);
                }
            }
        }
        else 
        {
                match x {

                'q' =>  {
                    terminate = true;
                },
                't' => {
                    app.show_table = true;
                    app.show_graphs = false
                }, 
                'g' => {
                    app.show_table = false;
                    app.show_graphs = true;
                }, 
                'd' => {
                    app.show_table = true;
                    app.show_graphs= true;
                },
                'k' =>  {
                    let _i = match app.state.selected() {
                        Some(i) => {
                            let pid_string  =&app.items[i][0] ;
                            let pid = pid_string.parse::<i32>().unwrap();
                            kill_process_and_children(pid as u32);

                        }
                        None => (),
                    };

                },
                _ => {}
                }  
        }, 

        KeyCode::Enter =>  {
            let _i = match app.state.selected() {
                Some(i) => {
                    let pid_string  =&app.items[i][0] ;
                    let pid = pid_string.parse::<i32>().unwrap();
                    app.oneP_ID = pid as u32;
                    app.show_single_process = !app.show_single_process;

                }
                None => (),
            };

        },
        
        KeyCode::Down => app.next(),
        KeyCode::Up => app.previous(),
        _ => {}
    }
    return terminate;
}