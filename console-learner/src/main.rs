mod app;
mod storage;
mod ui;

use app::App;

fn main() {
    let mut app = match App::new() {
        Ok(app) => app,
        Err(e) => {
            eprintln!("Failed to initialize application: {}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = app.run() {
        eprintln!("Application error: {}", e);
        std::process::exit(1);
    }

    println!("\nGoodbye! Keep learning! 🚀");
}

