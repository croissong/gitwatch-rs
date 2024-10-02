use anyhow::Result;
use fern::{
    colors::{Color, ColoredLevelConfig},
    Dispatch,
};
use log::Level;

use crate::cli::LogLevel;

pub fn setup_logger(level: LogLevel) -> Result<()> {
    let filter_level = level.into();

    let colors_line = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        // we actually don't need to specify the color for debug and info, they are white by default
        .info(Color::White)
        .debug(Color::Cyan)
        // depending on the terminals color scheme, this is the same as the background color
        .trace(Color::BrightBlack);

    let base_config = Dispatch::new()
        .format(move |out, message, record| {
            let level = &record.level();
            let color = colors_line.get_color(level).to_fg_str();
            let color_prefix = format!("\x1B[{color}m", color = color);
            const COLOR_SUFFIX: &str = "\x1B[0m";
            let level_prefix = if level == &Level::Info {
                "".to_string()
            } else {
                format!("[{level}] ", level = level.to_string().to_lowercase())
            };
            out.finish(format_args!(
                "{color_prefix}{level_prefix}{message}{COLOR_SUFFIX}",
                color_prefix = color_prefix,
                level_prefix = level_prefix,
                message = message,
                COLOR_SUFFIX = COLOR_SUFFIX
            ));
        })
        .level(filter_level)
        .level_for("ignore::gitignore", log::LevelFilter::Warn)
        .level_for("globset", log::LevelFilter::Warn);

    let stdout_config = Dispatch::new()
        .filter(|metadata| metadata.level() == log::Level::Info)
        .chain(std::io::stdout());

    let stderr_config = Dispatch::new()
        .filter(|metadata| metadata.level() != log::Level::Info)
        .chain(std::io::stderr());

    base_config
        .chain(stdout_config)
        .chain(stderr_config)
        .apply()?;

    Ok(())
}
