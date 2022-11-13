use yansi::Color;

pub fn hello() {
    let shell_color = Color::Yellow;
    let crab_color = Color::Red;
    let eye_color = Color::RGB(255, 255, 255);
    let ground_color = Color::White;
    let str_color = Color::Green;

    println!(
        r#"     {}
    {}       Welcome to {}!
    {}     The (very WIP) exotic shell.
    {}    Type {} for more instructions.
    {} {}{}
{}"#,
        shell_color.paint("__"),
        shell_color.paint("(__)_"),
        shell_color.paint("Crust"),
        shell_color.paint("(____)_"),
        shell_color.paint("(______)"),
        str_color.paint("'help'"),
        crab_color.paint("//("),
        eye_color.paint("00"),
        crab_color.paint(r#")\"#),
        ground_color.paint("................."),
    );
}
