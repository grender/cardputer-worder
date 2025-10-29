/// Display the main menu
pub fn display_menu() {
    println!("\n╔══════════════════════════════════════╗");
    println!("║     Vocabulary Learning System        ║");
    println!("╠══════════════════════════════════════╣");
    println!("║  1. Add new word                      ║");
    println!("║  2. Review due cards                  ║");
    println!("║  3. List all words                    ║");
    println!("║  4. Statistics                        ║");
    println!("║  5. Quit                              ║");
    println!("╚══════════════════════════════════════╝\n");
}

/// Read the user's menu choice
pub fn read_choice() -> String {
    print!("Select an option [1-5]: ");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap_or_default();
    input.trim().to_string()
}

/// Clear the screen (platform-specific)
pub fn clear_screen() {
    if cfg!(target_os = "windows") {
        std::process::Command::new("cmd")
            .args(&["/C", "cls"])
            .status()
            .ok();
    } else {
        std::process::Command::new("clear")
            .status()
            .ok();
    }
}

/// Wait for user to press Enter
pub fn wait_for_enter() {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).ok();
}

/// Read a line of input from the user
pub fn read_line(prompt: &str) -> String {
    print!("{}", prompt);
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap_or_default();
    input.trim().to_string()
}

