use std::fmt::Write as _;

const EMBLEM: &str = r#"                  ╭────────────────────────────╮
                  │                            ▼
               ┌──────┐     ┌──────┐     ┌──────┐
               │██████│     │▓▓▓▓▓▓│     │▒▒▒▒▒▒│
               │██████│     │▓▓▓▓▓▓│     │▒▒▒▒▒▒│
               └──────┘     └──────┘     └──────┘
                  ▲                            │
                  ╰────────────────────────────╯"#;

pub fn print(role: &str) {
    print!("{}", render(role));
}

fn render(role: &str) -> String {
    let mut banner = String::with_capacity(EMBLEM.len() + role.len() + 64);
    writeln!(banner, "{EMBLEM}\n").expect("writing to a string cannot fail");
    writeln!(
        banner,
        "          Homeostat :: {role} :: v{}\n",
        env!("CARGO_PKG_VERSION")
    )
    .expect("writing to a string cannot fail");
    banner
}

#[cfg(test)]
mod tests {
    use super::render;

    #[test]
    fn banner_identifies_the_role_and_version() {
        let banner = render("controller");

        assert!(banner.contains("Homeostat :: controller :: v0.1.0"));
        assert!(banner.contains("██████"));
    }
}
