use std::io::IsTerminal;

fn enabled() -> bool {
    std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none()
}

fn paint(s: &str, code: &str) -> String {
    if enabled() {
        format!("\x1b[{}m{}\x1b[0m", code, s)
    } else {
        s.to_string()
    }
}

pub fn success_line(label: &str, value: &str) -> String {
    let mark = paint("✓", "32");
    let label = paint(label, "1;32");
    format!("{} {} {}", mark, label, value)
}

pub fn info_line(label: &str, value: &str) -> String {
    let mark = paint("•", "36");
    let label = paint(label, "1;36");
    format!("{} {} {}", mark, label, value)
}

pub fn table_header(a: &str, b: &str, c: Option<&str>) -> String {
    let a = paint(a, "1");
    let b = paint(b, "1");
    match c {
        Some(c) => format!("{:<30} {:<15} {}", a, b, paint(c, "1")),
        None => format!("{:<15} {}", a, b),
    }
}
