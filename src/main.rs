use sysinfo::{System, SystemExt};
use std::env;


use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
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
    Terminal,
};


mod helperFunctions;
use helperFunctions::{print_process_tree, handle_key};
mod apps;
use apps::App;
mod UI;
use UI::ui;


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
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                
                if handle_key(key, &mut app) {
                    return Ok(())
                }
            }
        }
        
        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }
    }
}