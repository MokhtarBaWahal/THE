use sysinfo::{CpuExt,  ProcessExt,System, SystemExt, PidExt, ProcessStatus, UserExt};


use tui::widgets::TableState;

use nix::libc::{getpriority, id_t, PRIO_PROCESS};
use std::ffi::CString;

use std::time::Duration;
pub struct App  {
    
    pub system: System,
    pub data_cpu_avg: Vec<(f64, f64)>,
    pub data_cpus:    Vec<f32>,
    pub data_mem:     Vec<(f64, f64)>,
    pub data_swap:    Vec<(f64, f64)>,
    pub x: f64,
    pub time: f64,
    pub window: [f64; 2],
    pub state: TableState,
    pub items: Vec<Vec<String>>,
    pub sort_by_what: i32, 
    pub show_table: bool,
    pub show_graphs: bool,
    pub show_single_process: bool,
    pub oneP_ID: u32,
    pub user_n: String, 
   
}

impl App  {

    pub fn new(tick_rate: Duration,sys: &System, args: String) -> App  {
        
        let mut data_cpu_avg  = Vec::<(f64, f64)>::new();
        let mut data_mem     = Vec::<(f64, f64)>::new();
        let mut data_swap = Vec::<(f64, f64)>::new();

        let mut data_cpus: Vec<f32>= Vec::new();

        let mut system = System::new_all();
        system.refresh_all();
        for _processor in system.cpus() {
            data_cpus.push(0.0);
        }

        let x=200.0;
        for i in 0..200 {
            data_cpu_avg.push ((i as f64, 0.0));
            data_mem.push((i as f64, 0.0));
            data_swap.push((i as f64, 0.0));
        }

        let time = 35.0*(tick_rate.as_secs_f64() as f64 );

        let mut items = Vec::<Vec<String>>::new();

        // let s = System::new_all();
        for (pid, process) in sys.processes() {
            let pid_string = String::from(pid.as_u32().to_string());
            let parent_id = process.parent();
            let parent_pid_string = match parent_id {
                Some(id) => id.as_u32().to_string(),
                None => String::from(""),
            };
            //let priority = (process.cpu_usage() * 100.0) as i32;
            //let nice = -priority as i32;
            let virtual_memory = process.virtual_memory() / 1024;
            let memory = process.memory() / 1024;
            let shared_memory = (virtual_memory - memory) / 1024;
            let state = match process.status() {
                ProcessStatus::Run => "R",
                ProcessStatus::Sleep => "S",
                ProcessStatus::Idle => "I",
                ProcessStatus::Zombie => "Z",
                _ => "U",
            };
            let total_memory = system.total_memory();
            let mem_percent = (memory as f32 / total_memory as f32) * 100000.0;
            let mem_usage_str = format!("{:.2}%", mem_percent);
            let cpu_usage = process.cpu_usage();
            let cpu_usage_str = format!("{:.2}%", cpu_usage);
            let cpu_time = process.run_time() * 100;
            //let process_u = Process::new(pid_string.parse::<u32>().unwrap()).unwrap();
            //let user = process_u.username();
            let uid = process.user_id();
            //let user_name = self.system.get_user_by_id(uid);
            let user_name = system.get_user_by_id(uid.unwrap()).unwrap();
            // let user: Option<User> = get_user_by_uid(uid);
            // let user_f = "unkown";
            // match user {
            //     Some(u) => user_f = u.name().to_string_lossy(),
            // }
            //Some(priority)
            
            let _cstr = CString::new("").unwrap();
            let pid_n = pid_string.parse::<u32>().unwrap();
            let priority = 20 + unsafe { getpriority(PRIO_PROCESS as u32, pid_n as id_t) };
            let nice = -(20 - priority);
            let tem_process: Vec<String> = vec![
                pid_string,
                user_name.name().to_string(),
                parent_pid_string,
                priority.to_string(),
                nice.to_string(),
                memory.to_string(),
                shared_memory.to_string(),
                state.to_string(),
                cpu_usage_str,
                mem_usage_str,

                cpu_time.to_string(),
                // process_u.username().to_string(),
                String::from(process.name()),
            ];
            items.push(tem_process);
        }
        
        // sort by pid
        items.sort_by(|a, b| {
            let a_pid: i32 = a[0].parse().unwrap_or(0);
            let b_pid: i32 = b[0].parse().unwrap_or(0);
            a_pid.cmp(&b_pid)
        });
        let sort_bby = 0;
        if args == "" {
            App {
                system,
                data_cpu_avg,
                data_cpus,
                data_mem,
                data_swap,
                x,
                time,
                window: [0.0, 200.0],
                state: TableState::default(),
                items,
                sort_by_what: sort_bby,
                show_table: true,
                show_graphs: true,
                show_single_process: false,
                oneP_ID: 0,
                user_n: args
                
    
            }
        }
        else{
            items = items.into_iter()
            .filter(|v| v.get(1)==Some(&args))
            .collect();
            App {
                system,
                data_cpu_avg,
                data_cpus,
                data_mem,
                data_swap,
                x,
                time,
                window: [0.0, 200.0],
                state: TableState::default(),
                items,
                sort_by_what: sort_bby,
                show_table: true,
                show_graphs: true,
                show_single_process: false,
                oneP_ID: 0,
                user_n: args,
                
    
            }

        }
        

    }

    
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
    
    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn on_tick(&mut self ) {

        for _ in 0..10{
            self.data_cpu_avg.remove(0);
            self.data_mem.remove(0);

            
        }
        
        
        self.system.refresh_all();
        let mut i=0;

        for cpu in self.system.cpus() {
            
            let mut new_usage = self.data_cpus[i]-cpu.cpu_usage();
            new_usage = new_usage.abs();
            self.data_cpus[i]= new_usage;
            i = i+1;
        }
        let sum: f32 = self.data_cpus.iter().sum();
        let mut last = self.data_cpu_avg[ self.data_cpu_avg.len()-1].1;
        let avg = sum as f64 / self.data_cpus.len() as f64;
        let mut factor = ((avg - last )as f64).abs()/10.0;
    

        if last > avg {
            factor = factor * -1.0;
        }
        
        let memory_usage_percentage = (self.system.used_memory() as f64 / self.system.total_memory() as f64) * 100.0;

        for _ in 0..10{
       
            last = last + factor;
          
            self.x +=1.0;
            self.data_cpu_avg.push((self.x , last));
            self.data_mem.push((self.x , memory_usage_percentage));
            self.window[0] += 1.0;
            self.window[1] += 1.0;

        }
 

        let mut items = Vec::<Vec<String>>::new();

        
        // let s = System::new_all();
        for (pid, process) in self.system.processes() {
            let pid_string = String::from(pid.as_u32().to_string());
            let parent_id = process.parent();
            let parent_pid_string = match parent_id {
                Some(id) => id.as_u32().to_string(),
                None => String::from(""),
            };
            //let priority = (process.cpu_usage() * 100.0) as i32;
            //let nice = -priority as i32;
            let virtual_memory = process.virtual_memory() / 1024;
            let memory = process.memory() / 1024;
            let shared_memory = (virtual_memory - memory) / 1024;
            let state = match process.status() {
                ProcessStatus::Run => "R",
                ProcessStatus::Sleep => "S",
                ProcessStatus::Idle => "I",
                ProcessStatus::Zombie => "Z",
                _ => "U",
            };
            let total_memory = self.system.total_memory();
            let mem_percent = (memory as f32 / total_memory as f32) * 100000.0;
            let mem_usage_str = format!("{:.2}%", mem_percent);
            let cpu_usage = process.cpu_usage();
            let cpu_usage_str = format!("{:.2}%", cpu_usage);
            let cpu_time = process.run_time() * 100;
            //let process_u = Process::new(pid_string.parse::<u32>().unwrap()).unwrap();
            //let user = process_u.username();
            let uid = process.user_id();
            //let user_name = self.system.get_user_by_id(uid);
            let user_name = self.system.get_user_by_id(uid.unwrap()).unwrap();
    
            let pid_n = pid_string.parse::<u32>().unwrap();
            let priority = 20 + unsafe { getpriority(PRIO_PROCESS as u32, pid_n as id_t) };
            let nice = -(20 - priority);
            //let mut process2 = Process::new(pid_string.parse::<u32>().unwrap()).unwrap();
            //let mut cpu_percent = process2.cpu_percent().unwrap();
            let tem_process: Vec<String> = vec![
                pid_string,
                user_name.name().to_string(),
                parent_pid_string,
                priority.to_string(),
                nice.to_string(),
                //nice.to_string(),
                //virtual_memory.to_string(),
                memory.to_string(),
                shared_memory.to_string(),
                state.to_string(),
                cpu_usage_str,
                //cpu_percent.to_string(),
                mem_usage_str,

                cpu_time.to_string(),
                // process_u.username().to_string(),
                String::from(process.name()),
            ];
            items.push(tem_process);
        }

        if self.sort_by_what==0 {
                        
            items.sort_by(|a, b| {
                let a_pid: i32 = a[0].parse().unwrap_or(0);
                let b_pid: i32 = b[0].parse().unwrap_or(0);
                a_pid.cmp(&b_pid)
            });

        } 
        else if self.sort_by_what == 1{
             items.sort_by(|a, b| a[1].cmp(&b[1]));
            //items.sort_by_key(|v| v[11].to_lowercase());
        }
        else if self.sort_by_what == 3{
            items.sort_by(|a, b| {
                let a_pid: i32 = a[3].parse().unwrap_or(0);
                let b_pid: i32 = b[3].parse().unwrap_or(0);
                b_pid.cmp(&a_pid)
            });
        }
        else if self.sort_by_what == 8{
            items.sort_by(|a, b| {
                let a_cpu_usage = a[8].trim_end_matches('%').parse::<f32>().unwrap_or(0.0);
                let b_cpu_usage = b[8].trim_end_matches('%').parse::<f32>().unwrap_or(0.0);
                b_cpu_usage.partial_cmp(&a_cpu_usage).unwrap_or(std::cmp::Ordering::Equal)
            });
            
        }
        else if self.sort_by_what == 9{
            items.sort_by(|a, b| {
                let a_cpu_usage = a[9].trim_end_matches('%').parse::<f32>().unwrap_or(0.0);
                let b_cpu_usage = b[9].trim_end_matches('%').parse::<f32>().unwrap_or(0.0);
                b_cpu_usage.partial_cmp(&a_cpu_usage).unwrap_or(std::cmp::Ordering::Equal)
            });
        }
        else if self.sort_by_what == 11{
            items.sort_by_key(|v| v[11].to_lowercase());

        }
      
        self.items = items;
    

    }
}