mod app;
mod pages;

use pages::Page;

fn main() -> cosmic::iced::Result {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "--help" | "-h" => {
                print_help(&args[0]);
                Ok(())
            }
            "--version" | "-v" => {
                println!("cosmic-applet-settings {}", env!("CARGO_PKG_VERSION"));
                Ok(())
            }
            page_name => {
                if let Some(page) = Page::from_cli_name(page_name) {
                    app::run_app(page)
                } else {
                    eprintln!("Unknown page: {page_name}");
                    eprintln!("Available pages:");
                    for page in Page::ALL {
                        eprintln!("  {}", page.cli_name());
                    }
                    eprintln!("\nUse --help for usage information");
                    std::process::exit(1);
                }
            }
        }
    } else {
        app::run_app(Page::ALL[0])
    }
}

fn print_help(program: &str) {
    println!("Unified settings app for custom COSMIC applets\n");
    println!("Usage: {} [PAGE]\n", program);
    println!("Pages:");
    for page in Page::ALL {
        println!("  {:16} Open {}", page.cli_name(), page.title());
    }
    println!();
    println!("Options:");
    println!("  --version, -v      Show version information");
    println!("  --help, -h         Show this help message");
    println!();
    println!("If no page is specified, the first page is shown.");
}
