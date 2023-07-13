use sysinfo::{Pid,  ProcessExt, SystemExt, PidExt};


use tui::{
    backend::{Backend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols,
    text::{Span, Spans},
    widgets::{Axis, Block, Borders, Chart,Table, Dataset,Cell, Row,  Paragraph, Wrap},
    Frame
};




use crate::App;

pub fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {

    if app.show_single_process {

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

        
        let _output = format!("PID: {}\n Name: {}\nStatus: {}\nMemory: {} bytes\nExecutable: {}\nRun time: {} seconds\nCommand: {:?}\nStart time: {} seconds\nCPU usage: {}%\nCurrent working directory: {}\nVirtual memory: {} bytes\nParent process: {:?}\nRoot directory: {}",
                            pid, name, status, memory, exe, run_time, cmd, start_time, cpu_usage, cwd, virtual_memory, parent, root);

   
    // Words made "loooong" to demonstrate line breaking.
    let s = " hhhhhhhhhhhhhhhhhhhh";
    let mut long_line = s.repeat(usize::from(size.width) / s.len() + 4);
    long_line.push('\n');

    let _block = Block::default().style(Style::default().bg(Color::Black).fg(Color::White));
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
        

       
    Spans::from(Span::styled(name, Style::default().fg(Color::Green))),
    Spans::from(Span::styled(status, Style::default().fg(Color::Green))),
    Spans::from(Span::styled(memory, Style::default().fg(Color::Green))),
    Spans::from(Span::styled(exe, Style::default().fg(Color::Green))),
    Spans::from(Span::styled(run_time, Style::default().fg(Color::Green))),
    Spans::from(Span::styled(cmd, Style::default().fg(Color::Green))),
    Spans::from(Span::styled(start_time, Style::default().fg(Color::Green))),
    Spans::from(Span::styled(cpu_usage, Style::default().fg(Color::Green))),
    Spans::from(Span::styled(cwd, Style::default().fg(Color::Green))),
    Spans::from(Span::styled(virtual_memory, Style::default().fg(Color::Green))),
    Spans::from(Span::styled(parent, Style::default().fg(Color::Green))),
    Spans::from(Span::styled(root, Style::default().fg(Color::Green))),
    ];

    let create_block = |title| {
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black).fg(Color::Cyan))
            .title(Span::styled(
                title,
                Style::default().add_modifier(Modifier::BOLD),
            ))
    };

    let paragraph = Paragraph::new(text)
        .style(Style::default().bg(Color::Black).fg(Color::Green))
        .block(create_block("Info about process with pid: ".to_owned() + &pid.to_string()))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, chunks[0]);
}

}

pub fn show_table<B: Backend>(f: &mut Frame<B>, app: &mut App) {

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



pub fn show_full_app<B: Backend>(f: &mut Frame<B>, app: &mut App) {

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
        Spans::from(Span::styled(" 'q' to exit, 'd' to veiw the table and graphs 't' to veiw the table only,  'g' graphs only, k' to kill the selected process." , Style::default().fg(Color::Green))),
        Spans::from(Span::styled("  'Enter' to view more information about the selected process, or return to the table." , Style::default().fg(Color::Green))),
        Spans::from(Span::styled("  Ctrl + followed by a character for quick search about processes " , Style::default().fg(Color::Green))),
        Spans::from(Span::styled("  For filtering Shift followed 'p': PID 'u': User 'n': Priority  'c': %CPU 'm': %MEM 't': Time in seconds" , Style::default().fg(Color::Green))),
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
