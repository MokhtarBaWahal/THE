use sysinfo::{CpuExt,Pid,  ProcessExt,System, SystemExt, PidExt, ProcessStatus, UserExt};
use psutil::process::Process;

use std::thread::sleep;
use std::env;
use humantime::parse_duration;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols,
    text::{Span, Spans},
    widgets::{Axis, Block, Borders, Chart,Table, Dataset,Cell, Row, TableState,  Paragraph, Wrap},
    Frame, Terminal,
};

use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid as nixPID;



// use crate::App::UI;

// mod AppUI;

struct App  {
    
    system: System,
    data_cpu_avg: Vec<(f64, f64)>,
    data_cpus:    Vec<f32>,
    data_mem:     Vec<(f64, f64)>,
    data_swap:    Vec<(f64, f64)>,
    x: f64,
    time: f64,
    window: [f64; 2],
    state: TableState,
    items: Vec<Vec<String>>,
    sort_by_what: i32, 
    show_table: bool,
    show_graphs: bool,
    show_single_process: bool,
    oneP_ID: u32,
    user_n: String, 
   
}

impl App  {

    fn new(tick_rate: Duration,sys: &System, args: String) -> App  {
        
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
            use nix::libc::{getpriority, id_t, PRIO_PROCESS};
            use std::ffi::CString;
            let cstr = CString::new("").unwrap();
            let pid_n = pid_string.parse::<u32>().unwrap();
            let priority = 20 + unsafe { getpriority(PRIO_PROCESS as u32, pid_n as id_t) };
            let nice = -(20 - priority);
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


        
   

    fn on_tick(&mut self ) {

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
        let swap_usage_percentage = (self.system.used_memory() as f64 / self.system.total_memory() as f64) * 100.0;

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
            // let user: Option<User> = get_user_by_uid(uid);
            // let user_f = "unkown";
            // match user {
            //     Some(u) => user_f = u.name().to_string_lossy(),
            // }
            //Some(priority)
            use nix::libc::{getpriority, id_t, PRIO_PROCESS};
            use std::ffi::CString;
            let cstr = CString::new("").unwrap();
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
        else if self.sort_by_what == 2{
            items.sort_by(|a, b| {
                let a_pid: i32 = a[3].parse().unwrap_or(0);
                let b_pid: i32 = b[3].parse().unwrap_or(0);
                b_pid.cmp(&a_pid)
            });
        }
        else if self.sort_by_what == 3{
            items.sort_by(|a, b| {
                let a_pid: i32 = a[4].parse().unwrap_or(0);
                let b_pid: i32 = b[4].parse().unwrap_or(0);
                b_pid.cmp(&a_pid)
            });
        }
        else if self.sort_by_what == 4{
            items.sort_by(|a, b| {
                let a_pid: i32 = a[5].parse().unwrap_or(0);
                let b_pid: i32 = b[5].parse().unwrap_or(0);
                b_pid.cmp(&a_pid)
            });
        }
        else if self.sort_by_what == 5{
            items.sort_by(|a, b| {
                let a_pid: i32 = a[6].parse().unwrap_or(0);
                let b_pid: i32 = b[6].parse().unwrap_or(0);
                b_pid.cmp(&a_pid)
            });
        }
        else if self.sort_by_what == 6{
            // items.sort_by(|a, b| {
            //     let a_pid: i32 = a[7].parse().unwrap_or(0);
            //     let b_pid: i32 = b[7].parse().unwrap_or(0);
            //     b_pid.cmp(&a_pid)
            // });
        }
        else if self.sort_by_what == 7{
            items.sort_by(|a, b| {
                let a_state = a[7].chars().next();
                let b_state = b[7].chars().next();
                a_state.partial_cmp(&b_state).unwrap()
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
        else if self.sort_by_what == 10{
            items.sort_by(|a, b| {
                let a_time = parse_duration(&a[10]).unwrap_or_else(|_| Duration::from_secs(0)).as_secs();
                let b_time = parse_duration(&b[10]).unwrap_or_else(|_| Duration::from_secs(0)).as_secs();
                b_time.cmp(&a_time)
            });
        }
        else if self.sort_by_what == 11{
            items.sort_by(|a, b| b[12].cmp(&a[12]));
        }  
        else {
            
            items.sort_by_key(|v| v[11].to_lowercase());

        }
      
        self.items = items;
    

    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let mut user_selected ="";

     if args.len()>1 {
        if args[1]=="pstree"{
            let sys = System::new_all();
            let tick_rate = Duration::from_millis(2000);
            let app = App::new(tick_rate, &sys, user_selected.to_string());
         
            print_process_tree(&app.items, "1".to_string(), 0);
            return Ok(())
        }
        if args[1]=="u" && args.len()>2{
            
            user_selected= &args[2];
            
        }

    } 
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // // create app and run it
    let mut sys = System::new_all();
    sys.refresh_all();

    let tick_rate = Duration::from_millis(2000);
    let app = App::new(tick_rate, &sys, user_selected.to_string());
    let res = run_app(&mut terminal, app, tick_rate);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    let mut s_pressed = false;
    let mut f_pressed = false;
    let mut last_searched_letter: Option<char> = None;
    let mut selected_index: Option<usize> = None;
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('t') => {
                        app.show_table = true;
                        app.show_graphs = false
                    }, 
                    KeyCode::Char('g') => {
                        app.show_table = false;
                        app.show_graphs = true;
                    }, 
                    KeyCode::Char('d') => {
                        app.show_table = true;
                        app.show_graphs= true;
                    } 
                    // KeyCode::Char('p') =>  {
                    //     app.items.sort_by(|a, b| {
                    //         let a_pid: i32 = a[0].parse().unwrap_or(0);
                    //         let b_pid: i32 = b[0].parse().unwrap_or(0);
                    //         a_pid.cmp(&b_pid)
                    //     });
                    //     app.sort_by_what = 0;

                    // },
                    // KeyCode::Char('n') =>  {
                    //     app.items.sort_by_key(|v| v[11].to_lowercase());
                    //     app.sort_by_what = 1;

                    // },
                    KeyCode::Char('k') =>  {
                        let i = match app.state.selected() {
                            Some(i) => {
                                let pid_string  =&app.items[i][0] ;
                                let pid = pid_string.parse::<i32>().unwrap();
                                kill_process_and_children(pid as u32);

                            }
                            None => (),
                        };

                    },

                    KeyCode::Char('s') => {
                        s_pressed = true;
                    },

                    KeyCode::Char('f') => {
                        f_pressed = true;
                    
                    },
                    KeyCode::Char(c) if f_pressed => {
                        if c == 'p'{
                            app.sort_by_what = 0;
                        }
                        else if c == 'u'{
                            app.sort_by_what = 1;

                        }
                        else if c == 'r'{
                            app.sort_by_what = 2;
                        }
                        else if c == 'n'{
                            app.sort_by_what = 3;

                        }
                        else if c == 'v'{
                            app.sort_by_what = 4;

                        }
                        else if c == 'e'{
                            app.sort_by_what = 5;

                        }
                        else if c == 'h'{
                            app.sort_by_what = 6;

                        }
                        else if c == 'b'{
                            app.sort_by_what = 7;

                        }
                        else if c == 'c'{ 
                            app.sort_by_what = 8;

                        }
                        else if c == 'm'{
                            app.sort_by_what = 9;

                        }
                        else if c == 't'{
                            app.sort_by_what = 10;

                        }
                        // else if c == 'x'{
                        //     app.sort_by_what = 11;

                        // }
                        
                        f_pressed = false;
                    }

                    KeyCode::Char(c) if s_pressed => {
                        let mut start_index = 0;
                        if let Some(index) = selected_index {
        // If there is a recorded index, start the search from the next process after the last selected process
                        start_index = index + 1;
                        }   
                        // Search for the next process starting with 'a'
                        if let Some(index) = app.items.iter().skip(start_index).position(|row| row[11].starts_with(c)) {
                            // Do something with the matching process

                            // Set the selected index to the index of the matching process
                            app.state.select(Some(start_index + index));
                            // Record the index of the last selected process
                            selected_index = Some(start_index + index);
                        } else {
                            // If there are no more matches, start the search again from the beginning of the process list
                            if let Some(index) = app.items.iter().position(|row| row[11].starts_with(c)) {
                                // Do something with the matching process

                                // Set the selected index to the index of the matching process
                                app.state.select(Some(index));
                                // Record the index of the last selected process
                                selected_index = Some(index);
                            }
                        }
                                            s_pressed = false;
                    },



                    KeyCode::Enter =>  {
                        let i = match app.state.selected() {
                            Some(i) => {
                                let pid_string  =&app.items[i][0] ;
                                let pid = pid_string.parse::<i32>().unwrap();
                                app.oneP_ID = pid as u32;
                                app.show_single_process = !app.show_single_process;

                            }
                            None => (),
                        };
                        
                        
                        // print_process_tree(&app.items, "1".to_string(), 0);

                    },
                    
                    KeyCode::Down => app.next(),
                    KeyCode::Up => app.previous(),
                    _ => {}
                }
            }
        }

        
        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {

    if (app.show_single_process){

        show_one_process(f, app);
        
    } else {

        if (app.show_graphs && app.show_table) || (!app.show_graphs && !app.show_table){

            show_full_app(f, app)  ;
    
        }
        if app.show_graphs && !app.show_table  {
    
           show_graphs_only(f, app);
            
        }
    
        if !app.show_graphs && app.show_table {
            
            show_table(f, app); 
            
    
        }

    }





}



fn kill_process_and_children(pid: u32) {
    let mut system = System::new();
    //let pid = Pid::from_raw(pid);
    for (pid2, process) in system.processes() {
        if process.parent() == Some(Pid::from_u32(pid)) {
            kill_process_and_children(process.pid().as_u32());
        }
    }
     let pid = nixPID::from_raw(pid as i32);
    kill(pid, Signal::SIGTERM);


}


fn print_process_tree(items:  &[Vec<String>], id: String,  depth: usize) {
    let padding = "  ".repeat(depth);
    let result = items.iter().find(|&v| v[0] == id);
    let mut p_name = "";
    if let Some(vec) = result {
        if let Some(name) = vec.get(12) {
            p_name= name;
        }
    }
    if(id=="1"){

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


fn show_full_app<B: Backend>(f: &mut Frame<B>, app: &mut App) {

    let size = f.size();
 
    
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 6)
            ]
            .as_ref(),
        )
        .split(size);


    let chunks_left = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3)
                
            ]
            .as_ref(),
        )
        .split(chunks[0]);
    let x_labels = vec![
        Span::styled(
            format!("{}", app.time.to_string() + "s"),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{}", "0"),
            Style::default().add_modifier(Modifier::BOLD),
        ),
    ];
    let datasets = vec![
        Dataset::default()
            .name("CPU Usage",)
            .marker(symbols::Marker::Dot)
            .style(Style::default().fg(Color::Red))
            .data(&app.data_cpu_avg),
        
    ];

    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .title(Span::styled(
                    "CPU usage",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL).style(Style::default().fg(Color::Cyan)),
        )
        .x_axis(
            Axis::default()
            .title(Span::styled("Time", Style::default().fg(Color::Green)))
                .style(Style::default().fg(Color::Cyan))
                .labels(x_labels)
                .bounds(app.window),
        )
        .y_axis(
            Axis::default()
            .title(Span::styled("Percentage", Style::default().fg(Color::Green)))
                .style(Style::default().fg(Color::Cyan))
                .labels(vec![
                    Span::raw("0"),
                    Span::styled("25", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled("50", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled("75", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled("100", Style::default().add_modifier(Modifier::BOLD)),
                ])
                .bounds([0.0, 100.0]),
        );
    f.render_widget(chart, chunks_left[1]);


    
    let x_labels = vec![
        Span::styled(
            format!("{}", app.time.to_string() + "s"),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{}", "0"),
            Style::default().add_modifier(Modifier::BOLD),
        ),
    ];
    let datasets = vec![
        Dataset::default()
            .name("CPU Usage",)
            .marker(symbols::Marker::Dot)
            .style(Style::default().fg(Color::Blue))
            .data(&app.data_mem),
        
    ];

    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .title(Span::styled(
                    "Memory usage",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL).style(Style::default().fg(Color::Cyan)),
        )
        .x_axis(
            Axis::default()
            .title(Span::styled("Time", Style::default().fg(Color::Green)))
                .style(Style::default().fg(Color::Cyan))
                .labels(x_labels)
                .bounds(app.window),
        )
        .y_axis(
            Axis::default()
            .title(Span::styled("Percentage", Style::default().fg(Color::Green)))
                .style(Style::default().fg(Color::Cyan))
                .labels(vec![
                    Span::raw("0"),
                    Span::styled("25", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled("50", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled("75", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled("100", Style::default().add_modifier(Modifier::BOLD)),
                ])
                .bounds([0.0, 100.0]),
        );
    f.render_widget(chart, chunks_left[2]);
    let chunks_right = Layout::default()
    .direction(Direction::Vertical)
    .constraints(
        [
            Constraint::Percentage(80),
            Constraint::Percentage(20),
            
            
        ]
        .as_ref(),
    )
    .split(chunks[1]);  
    let rects = Layout::default()
        .constraints([Constraint::Percentage(50)].as_ref())
        .margin(0)  
        .split(chunks_right[0]);

    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default().bg(Color::Blue);
    let header_cells = ["PID", "USER", "PTID", "PR", "NI",  "RES", "SHR", "S", "%CPU", "%MEM", "TIME+", "COMMAND"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red)));
    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(1);
    // let table_data =
    let rows = app.items.iter().map(|item| {
        let height = item
            .iter()
            .map(|content| content.chars().filter(|c| *c == '\n').count())
            .max()
            .unwrap_or(0)
            + 1;
        let cells = item.iter().map(|c| Cell::from(&**c));
        Row::new(cells).height(height as u16).bottom_margin(1)
    });
    let t = Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Table").style(Style::default().fg(Color::Cyan)))
        .highlight_style(selected_style)
        .highlight_symbol("")
        .widths(&[
            Constraint::Percentage(8),
            Constraint::Percentage(8),
            Constraint::Percentage(8),
            Constraint::Percentage(8),
            Constraint::Percentage(8),
            Constraint::Percentage(8),
            Constraint::Percentage(8),
            Constraint::Percentage(8),
            Constraint::Percentage(8),
            Constraint::Percentage(8),
            Constraint::Percentage(8),
            Constraint::Percentage(8),
        ]);
        
    f.render_stateful_widget(t, rects[0], &mut app.state);


    let name = if let Some (name ) = app.system.name() { name } else { todo!() };
    let kVerison = if let Some (kVerison ) = app.system.kernel_version() { kVerison } else { todo!() };
    let sys_name = format!("System name = {}", name);
    let num_cores = format!("Number of cores = {}", app.system.cpus().len());
    let num_disks = format!("Number of disks = {}", app.system.disks().len());
    let ker_ver = format!("Kernel version = {}", kVerison);
    let total_memory = format!("Total memory = {} bytes" , app.system.total_memory());
    let av_memory = format!("Available memory = {} bytes", app.system.available_memory());
    let free_memory = format!("Free memory \n = {} bytes", app.system.free_memory());
    let used_memory = format!("Used memory = {} bytes", app.system.used_memory());
    let num_process = format!("Number of processes = {} process", app.system.processes().len());
    let cpu_us = format!("CPU usage  = {:.3} %", app.data_cpu_avg[app.data_cpu_avg.len()-1].1);
    let mem_us = format!("Memory usage  = {:.3} %", app.data_mem[app.data_mem.len()-1].1);

    let text = vec![
        Spans::from(Span::styled(sys_name , Style::default().fg(Color::Green))),
        Spans::from(Span::styled(num_cores , Style::default().fg(Color::Green))),
        Spans::from(Span::styled(num_disks , Style::default().fg(Color::Green))),
        Spans::from(Span::styled(ker_ver , Style::default().fg(Color::Green))),
        Spans::from(Span::styled(total_memory , Style::default().fg(Color::Green))),
        Spans::from(Span::styled(av_memory , Style::default().fg(Color::Green))),
        Spans::from(Span::styled(free_memory , Style::default().fg(Color::Green))),
        Spans::from(Span::styled(used_memory , Style::default().fg(Color::Green))),
        Spans::from(Span::styled(num_process, Style::default().fg(Color::Green))),
        Spans::from(Span::styled(cpu_us, Style::default().fg(Color::Green))),
        Spans::from(Span::styled(mem_us, Style::default().fg(Color::Green))),

    ];

    let create_block = |title| {
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default())
            .title(Span::styled(
                title,
                Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan),
            ))
    };

    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(Color::Cyan))
        .block(create_block("Info about  system."))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, chunks_left[0]);

    let text = vec![
        // Spans::from(Span::styled("HOTKEYS" , Style::default().fg(Color::Green))),
        // Spans::from(Span::styled("  " , Style::default().fg(Color::Green))),
        Spans::from(Span::styled(" 'q' to exit. 'd' to veiw the table and graphs 't' to veiw the table only 'g' graphs only." , Style::default().fg(Color::Green))),
        Spans::from(Span::styled("  'Enter' to view more information about the selected process." , Style::default().fg(Color::Green))),
        Spans::from(Span::styled("  'k' to kill the selected process." , Style::default().fg(Color::Green))),
        Spans::from(Span::styled("  's' followed by a character for quick search" , Style::default().fg(Color::Green))),
        Spans::from(Span::styled("  For filtering 'f' followed 'p': PID 'u': User 'r': Priority 'n': Niceness 'e': Memory 'h': Shared Memory 'b': State 'c': %CPU 'm': %MEM 't': Time in seconds" , Style::default().fg(Color::Green))),
        Spans::from(Span::styled("  './app pstree' to print tree." , Style::default().fg(Color::Green))),
        Spans::from(Span::styled("  './app u (username)' to get selected user processes only." , Style::default().fg(Color::Green))),
        
  
    ];

    let create_block = |title| {
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default())
            .title(Span::styled(
                title,
                Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan),
            ))
    };

    let paragraph = Paragraph::new(text)
        .style(Style::default())
        .block(create_block("Help how to use THE"))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, chunks_right[1]);


}



fn show_graphs_only<B: Backend>(f: &mut Frame<B>, app: &mut App) {

    let size = f.size();
    let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints(
        [
            Constraint::Ratio(1, 2),
            Constraint::Ratio(1, 2)
        ]
        .as_ref(),
    )
    .split(size);



    let x_labels = vec![
        Span::styled(
            format!("{}", app.time.to_string() + "s"),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{}", "0"),
            Style::default().add_modifier(Modifier::BOLD),
        ),
    ];
    let datasets = vec![
        Dataset::default()
            .name("CPU Usage",)
            .marker(symbols::Marker::Dot)
            .style(Style::default().fg(Color::Red))
            .data(&app.data_cpu_avg),
        
    ];

    let chart = Chart::new(datasets)
        .block(
            Block::default().style(Style::default().bg(Color::Red))
                .title(Span::styled(
                    "CPU usage",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL).style(Style::default().fg(Color::Cyan)),
        )
        .x_axis(
            Axis::default()
            .title(Span::styled("Time", Style::default().fg(Color::Green)))
                .style(Style::default().fg(Color::Red))
                .labels(x_labels)
                .bounds(app.window),
        )
        .y_axis(
            Axis::default()
            .title(Span::styled("Percentage", Style::default().fg(Color::Green)))
                .style(Style::default().fg(Color::Cyan))
                .labels(vec![
                    Span::raw("0"),
                    Span::styled("100", Style::default().add_modifier(Modifier::BOLD)),
                ])
                .bounds([0.0, 100.0]),
        );
    f.render_widget(chart, chunks[0]);


    let x_labels = vec![
        Span::styled(
            format!("{}", app.time.to_string() + "s"),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{}", "0"),
            Style::default().add_modifier(Modifier::BOLD),
        ),
    ];
    let datasets = vec![
        Dataset::default()
            .name("Memory Usage",)
            .marker(symbols::Marker::Dot)
            .style(Style::default().fg(Color::Blue))
            .data(&app.data_mem),
        
    ];

    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .title(Span::styled(
                    "Memory usage",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL).style(Style::default().fg(Color::Cyan)),
        )
        .x_axis(
            Axis::default()
                .title(Span::styled("Time", Style::default().fg(Color::Green)))
                .style(Style::default().fg(Color::Cyan))
                .labels(x_labels)
                .bounds(app.window),
        )
        .y_axis(
            Axis::default()
            .title(Span::styled("Percentage", Style::default().fg(Color::Green)))
                .style(Style::default().fg(Color::Cyan))
                .labels(vec![
                    Span::raw("0"),
                    Span::styled("100", Style::default().add_modifier(Modifier::BOLD)),
                ])
                .bounds([0.0, 100.0]),
        );
    f.render_widget(chart, chunks[1]);

}


fn show_table<B: Backend>(f: &mut Frame<B>, app: &mut App) {

    let size = f.size();
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Ratio(1, 1)
            ]
            .as_ref(),
        )
        .split(size);



    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default().bg(Color::Blue);
    let header_cells = ["PID", "USER", "PTID", "PR", "NI", "RES", "SHR", "S", "%CPU", "%MEM", "TIME+", "COMMAND"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red)));
    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(1);
    let rows = app.items.iter().map(|item| {
        let height = item
            .iter()
            .map(|content| content.chars().filter(|c| *c == '\n').count())
            .max()
            .unwrap_or(0)
            + 1;
        let cells = item.iter().map(|c| Cell::from(&**c));
        Row::new(cells).height(height as u16).bottom_margin(1)
    });
    let t = Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Table").style(Style::default().fg(Color::Cyan)))
        .highlight_style(selected_style)
        .highlight_symbol("")
        .widths(&[
            Constraint::Percentage(8),
            Constraint::Percentage(8),
            Constraint::Percentage(8),
            Constraint::Percentage(8),
            Constraint::Percentage(8),
            Constraint::Percentage(8),
            Constraint::Percentage(8),
            Constraint::Percentage(8),
            Constraint::Percentage(8),
            Constraint::Percentage(8),
            Constraint::Percentage(8),
            Constraint::Percentage(8),
        ]);
    f.render_stateful_widget(t, chunks[0], &mut app.state);

}

fn show_one_process <B: Backend>(f: &mut Frame<B>, app: &mut App){

    let size = f.size(); 
    let pid = app.oneP_ID;
    if let Some(process) = app.system.process(Pid::from_u32(pid)) {

        let name = format!("Name: {:?}",process.name());
        let status = format!("Status: {:?}", process.status());
        let memory = format!("Memory: {:?} bytes", process.memory());
        let exe = format!("Executable: {:?} ", process.exe().display());
        let run_time = format!("Run time: {:?} seconds", process.run_time());
        let cmd = format!("Command{:?}", process.cmd());
        let start_time = format!("Start time: {:?} seconds",process.start_time());
        let cpu_usage = format!("CPU usage: {:?} %",process.cpu_usage());
        let cwd = format!("Current working directory: {:?}",process.cwd().display());
        let virtual_memory = format!("Virtual memory: {:?}", process.virtual_memory());
        let parent = format!("Parent process: {:?}", process.parent());
        let root = format!("Root: {:?}", process.root().display());

        
        let output = format!("PID: {}\n Name: {}\nStatus: {}\nMemory: {} bytes\nExecutable: {}\nRun time: {} seconds\nCommand: {:?}\nStart time: {} seconds\nCPU usage: {}%\nCurrent working directory: {}\nVirtual memory: {} bytes\nParent process: {:?}\nRoot directory: {}",
                            pid, name, status, memory, exe, run_time, cmd, start_time, cpu_usage, cwd, virtual_memory, parent, root);

   
    // Words made "loooong" to demonstrate line breaking.
    let s = " hhhhhhhhhhhhhhhhhhhh";
    let mut long_line = s.repeat(usize::from(size.width) / s.len() + 4);
    long_line.push('\n');

    let block = Block::default().style(Style::default().bg(Color::White).fg(Color::Black));
    // f.render_widget(block, size);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(5)
        .constraints(
            [
                Constraint::Percentage(100),
                
            ]
            .as_ref(),
        )
        .split(size);

    let text = vec![
        

       
    Spans::from(Span::styled(name, Style::default().bg(Color::Green))),
    Spans::from(Span::styled(status, Style::default().bg(Color::Green))),
    Spans::from(Span::styled(memory, Style::default().bg(Color::Green))),
    Spans::from(Span::styled(exe, Style::default().bg(Color::Green))),
    Spans::from(Span::styled(run_time, Style::default().bg(Color::Green))),
    Spans::from(Span::styled(cmd, Style::default().bg(Color::Green))),
    Spans::from(Span::styled(start_time, Style::default().bg(Color::Green))),
    Spans::from(Span::styled(cpu_usage, Style::default().bg(Color::Green))),
    Spans::from(Span::styled(cwd, Style::default().bg(Color::Green))),
    Spans::from(Span::styled(virtual_memory, Style::default().bg(Color::Green))),
    Spans::from(Span::styled(parent, Style::default().bg(Color::Green))),
    Spans::from(Span::styled(root, Style::default().bg(Color::Green))),
    ];

    let create_block = |title| {
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::White).fg(Color::Black))
            .title(Span::styled(
                title,
                Style::default().add_modifier(Modifier::BOLD),
            ))
    };

    let paragraph = Paragraph::new(text)
        .style(Style::default().bg(Color::White).fg(Color::Black))
        .block(create_block("Info about process with pid: ".to_owned() + &pid.to_string()))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, chunks[0]);
}

}
