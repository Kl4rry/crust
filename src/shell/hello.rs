use crossterm::style::{Color, Stylize};

pub fn hello() {
    let shell_color = Color::Yellow;
    let crab_color = Color::Red;
    let eye_color = Color::Rgb {
        r: 255,
        g: 255,
        b: 255,
    };
    let ground_color = Color::White;
    let str_color = Color::Green;

    println!(
        r#"     {}
    {}       Welcome to {}!
    {}     The (very WIP) exotic shell.
    {}    Type {} for more instructions.
    {} {}{}
{}"#,
        ("__").with(shell_color),
        ("(__)_").with(shell_color),
        ("Crust").with(shell_color),
        ("(____)_").with(shell_color),
        ("(______)").with(shell_color),
        ("'help'").with(str_color),
        ("//(").with(crab_color),
        ("00").with(eye_color),
        (r")\").with(crab_color),
        (".................").with(ground_color),
    );
}
