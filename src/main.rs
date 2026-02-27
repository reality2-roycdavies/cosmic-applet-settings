mod app;
mod detection;
mod pages;

use app::AppFlags;
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
                    let active_pages = detection::detect_active_pages();
                    app::run_app(AppFlags {
                        initial_page: page,
                        active_pages,
                    })
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
        let active_pages = detection::detect_active_pages();
        let initial_page = active_pages.first().copied().unwrap_or(Page::ALL[0]);
        app::run_app(AppFlags {
            initial_page,
            active_pages,
        })
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
    println!("If no page is specified, the first active page is shown.");
    println!("Only pages for applets on the panel or dock are displayed.");
}
