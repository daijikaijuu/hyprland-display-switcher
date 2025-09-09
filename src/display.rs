use crate::config::{ConfigManager, ExtendConfiguration, ExtendLayout};
use hyprland::data::Monitor;
use hyprland::dispatch::{Dispatch, DispatchType};
use std::process::Command;

pub fn determine_primary_monitor<'a>(
    monitors: &'a [Monitor],
    config_manager: &ConfigManager,
) -> &'a Monitor {
    let monitor_names: Vec<String> = monitors.iter().map(|m| m.name.clone()).collect();

    if let Some(preferred_primary) = config_manager.get_preferred_primary_monitor(&monitor_names)
        && let Some(monitor) = monitors.iter().find(|m| m.name == preferred_primary)
    {
        return monitor;
    }

    // Fallback: use focused monitor first, then first monitor
    monitors.iter().find(|m| m.focused).unwrap_or(&monitors[0])
}

fn determine_secondary_monitor<'a>(
    monitors: &'a [Monitor],
    primary: &Monitor,
) -> Option<&'a Monitor> {
    monitors.iter().find(|m| m.name != primary.name)
}

pub fn apply_mirror_mode(
    monitors: &[Monitor],
    config_manager: &crate::config::ConfigManager,
) -> Result<(), String> {
    if monitors.len() < 2 {
        return Ok(());
    }

    let primary_mon = determine_primary_monitor(monitors, config_manager);
    let secondary_mon = monitors
        .iter()
        .find(|m| m.name != primary_mon.name)
        .ok_or("Secondary monitor not found")?;

    // Configure primary monitor
    Dispatch::call(DispatchType::Exec(&format!(
        "hyprctl keyword monitor \"{},{}x{},0x0,{}\"",
        primary_mon.name, primary_mon.width, primary_mon.height, primary_mon.scale
    )))
    .map_err(|e| e.to_string())?;

    // Configure secondary monitor to mirror primary
    Dispatch::call(DispatchType::Exec(&format!(
        "hyprctl keyword monitor \"{},{}x{},0x0,{},mirror,{}\"",
        secondary_mon.name,
        primary_mon.width,
        primary_mon.height,
        primary_mon.scale,
        primary_mon.name
    )))
    .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn apply_extend_mode(monitors: &[Monitor], config: &ExtendConfiguration) -> Result<(), String> {
    if monitors.len() < 2 {
        return Ok(());
    }

    // Find monitors by name
    let primary_monitor = monitors
        .iter()
        .find(|m| m.name == config.primary_monitor)
        .ok_or("Primary monitor not found")?;

    let _secondary_monitor = monitors
        .iter()
        .find(|m| m.name == config.secondary_monitor)
        .ok_or("Secondary monitor not found")?;

    // Calculate positions based on resolutions and layout
    let primary_resolution = if config.primary_resolution == "auto" {
        format!("{}x{}", primary_monitor.width, primary_monitor.height)
    } else {
        config.primary_resolution.clone()
    };

    // Parse dimensions for positioning calculations
    let primary_width = if config.primary_resolution == "auto" {
        primary_monitor.width as i32
    } else {
        primary_resolution
            .split('x')
            .next()
            .unwrap_or("1920")
            .parse()
            .unwrap_or(1920)
    };

    let primary_height = if config.primary_resolution == "auto" {
        primary_monitor.height as i32
    } else {
        primary_resolution
            .split('x')
            .nth(1)
            .unwrap_or("1080")
            .parse()
            .unwrap_or(1080)
    };

    let (primary_pos, secondary_pos) = calculate_positions(
        &config.layout,
        primary_width,
        primary_height,
        &config.secondary_resolution,
    );

    // Build transform strings
    let primary_transform = get_transform_string(&config.primary_rotation);
    let secondary_transform = get_transform_string(&config.secondary_rotation);

    // Build commands
    let primary_command = format!(
        "hyprctl keyword monitor \"{},{}{}\"",
        config.primary_monitor,
        if config.primary_resolution == "auto" {
            format!("auto,{primary_pos},1")
        } else {
            format!("{},{},1", config.primary_resolution, primary_pos)
        },
        primary_transform
    );

    let secondary_command = format!(
        "hyprctl keyword monitor \"{},{},{},1{}\"",
        config.secondary_monitor, config.secondary_resolution, secondary_pos, secondary_transform
    );

    eprintln!("Primary command: {primary_command}");
    eprintln!("Secondary command: {secondary_command}");

    // Disable both monitors first to reset their state
    Dispatch::call(DispatchType::Exec(&format!(
        "hyprctl keyword monitor \"{},disable\"",
        config.primary_monitor
    )))
    .map_err(|e| e.to_string())?;
    Dispatch::call(DispatchType::Exec(&format!(
        "hyprctl keyword monitor \"{},disable\"",
        config.secondary_monitor
    )))
    .map_err(|e| e.to_string())?;

    // Wait for the changes to take effect
    std::thread::sleep(std::time::Duration::from_millis(1000));

    // Apply both monitor configurations
    Dispatch::call(DispatchType::Exec(&primary_command)).map_err(|e| e.to_string())?;
    Dispatch::call(DispatchType::Exec(&secondary_command)).map_err(|e| e.to_string())?;

    Ok(())
}

pub fn apply_single_screen_mode(
    monitors: &[Monitor],
    primary_only: bool,
    config_manager: &ConfigManager,
) -> Result<(), String> {
    if monitors.len() < 2 {
        return Ok(());
    }

    let primary_monitor = determine_primary_monitor(monitors, config_manager);
    let secondary_monitor = determine_secondary_monitor(monitors, primary_monitor)
        .ok_or("Secondary monitor not found")?;

    let (active_mon, inactive_mon) = if primary_only {
        (primary_monitor, secondary_monitor)
    } else {
        (secondary_monitor, primary_monitor)
    };

    // Enable active monitor with its native resolution
    Dispatch::call(DispatchType::Exec(&format!(
        "hyprctl keyword monitor \"{},{}x{},0x0,{}\"",
        active_mon.name, active_mon.width, active_mon.height, active_mon.scale
    )))
    .map_err(|e| e.to_string())?;

    // Disable inactive monitor
    Dispatch::call(DispatchType::Exec(&format!(
        "hyprctl keyword monitor \"{},disable\"",
        inactive_mon.name
    )))
    .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn reset_to_defaults() -> Result<(), String> {
    Dispatch::call(DispatchType::Exec("hyprctl reload")).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_monitor_available_modes(monitor_name: &str) -> Vec<String> {
    let output = Command::new("hyprctl").args(["monitors", "all"]).output();

    match output {
        Ok(output) if output.status.success() => {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let modes = parse_monitor_modes(&output_str, monitor_name);
            eprintln!("Available modes for {monitor_name}: {modes:?}");
            modes
        }
        _ => {
            eprintln!("Failed to get modes for {monitor_name}, using fallback");
            // Fallback to common resolutions if hyprctl fails
            vec![
                "1920x1080".to_string(),
                "2560x1440".to_string(),
                "3840x2160".to_string(),
                "1680x1050".to_string(),
                "1366x768".to_string(),
                "1440x900".to_string(),
            ]
        }
    }
}

fn calculate_positions(
    layout: &ExtendLayout,
    primary_width: i32,
    primary_height: i32,
    secondary_resolution: &str,
) -> (String, String) {
    match layout {
        ExtendLayout::LeftToRight => ("0x0".to_string(), format!("{primary_width}x0")),
        ExtendLayout::RightToLeft => {
            let secondary_width = secondary_resolution
                .split('x')
                .next()
                .unwrap_or("1920")
                .parse::<i32>()
                .unwrap_or(1920);
            (format!("{secondary_width}x0"), "0x0".to_string())
        }
        ExtendLayout::TopToBottom => ("0x0".to_string(), format!("0x{primary_height}")),
        ExtendLayout::BottomToTop => {
            let secondary_height = secondary_resolution
                .split('x')
                .nth(1)
                .unwrap_or("1080")
                .parse::<i32>()
                .unwrap_or(1080);
            (format!("0x{secondary_height}"), "0x0".to_string())
        }
    }
}

fn get_transform_string(rotation: &str) -> &'static str {
    match rotation {
        "left" => ",transform,1",
        "right" => ",transform,3",
        "inverted" => ",transform,2",
        _ => "",
    }
}

fn parse_monitor_modes(output: &str, target_monitor: &str) -> Vec<String> {
    let mut modes = Vec::new();
    let mut in_target_monitor = false;

    for line in output.lines() {
        let trimmed = line.trim();

        // Check if we're entering the target monitor section
        if trimmed.starts_with(&format!("Monitor {target_monitor}")) {
            in_target_monitor = true;
            continue;
        }

        // Check if we're leaving the current monitor section
        if in_target_monitor
            && trimmed.starts_with("Monitor ")
            && !trimmed.starts_with(&format!("Monitor {target_monitor}"))
        {
            break;
        }

        // Look for the available modes section
        if in_target_monitor && trimmed.starts_with("availableModes:") {
            // Extract modes from the same line: "availableModes: 1920x1080@60.00Hz ..."
            let modes_str = trimmed.strip_prefix("availableModes:").unwrap_or("").trim();
            for mode_str in modes_str.split_whitespace() {
                if let Some(resolution) = extract_resolution_from_line(mode_str)
                    && !modes.contains(&resolution)
                {
                    modes.push(resolution);
                }
            }
            break; // We found the modes line, we're done
        }
    }

    // If no modes found, provide fallback
    if modes.is_empty() {
        modes = vec![
            "1920x1080".to_string(),
            "2560x1440".to_string(),
            "3840x2160".to_string(),
        ];
    }

    modes
}

fn extract_resolution_from_line(line: &str) -> Option<String> {
    // Look for pattern like "1920x1080@60.00" or "  1920x1080@60.00hz"
    if let Some(at_pos) = line.find('@') {
        let before_at = &line[..at_pos];
        if let Some(_x_pos) = before_at.rfind('x') {
            let resolution_part = before_at.trim();
            // Find the start of the resolution (numbers)
            let start = resolution_part
                .chars()
                .position(|c| c.is_ascii_digit())
                .unwrap_or(0);
            let resolution = &resolution_part[start..];
            if resolution.contains('x')
                && resolution.chars().all(|c| c.is_ascii_digit() || c == 'x')
            {
                return Some(resolution.to_string());
            }
        }
    }
    None
}
