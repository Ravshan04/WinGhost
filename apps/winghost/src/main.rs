use pty_windows::PlatformSupport;
use renderer::RenderSnapshot;
use terminal_core::Terminal;

fn main() {
    let terminal = Terminal::new(80, 24);
    let snapshot = RenderSnapshot::from_terminal(&terminal);

    println!(
        "WinGhost scaffold: {}x{} grid, platform support: {}",
        snapshot.columns(),
        snapshot.rows(),
        PlatformSupport::current()
    );
}
