use std::fs::{self, create_dir_all};
use std::path::Path;
use std::process::Command;

/// Save GraphViz DOT files to disk and generate visualizations
///
/// This function:
/// 1. Creates the output directory and subdirectory if they don't exist
/// 2. Saves the DOT file
/// 3. Runs the GraphViz 'dot' command to generate SVG visualization
/// 4. Uses ImageMagick to convert SVG to PNG (if available)
/// 5. Optionally opens the generated PNG files for viewing
///
/// # Arguments
/// * `dot_content` - GraphViz DOT language content as a string
/// * `name` - Base name for the generated files
/// * `viz_type` - Type of visualization (e.g., "network", "petri")
/// * `open_files` - Whether to open the generated PNG files for viewing
///
/// Returns a Result with the paths to the generated files or an error message
pub fn save_graphviz(
    dot_content: &str,
    name: &str,
    viz_type: &str,
    open_files: bool,
) -> Result<Vec<String>, String> {
    // Create main output directory if it doesn't exist
    let out_dir = Path::new("out");
    if let Err(e) = create_dir_all(out_dir) {
        return Err(format!("Failed to create output directory: {}", e));
    }

    // Create subdirectory for this specific output
    let subdir_name = name;
    let subdir_path = out_dir.join(subdir_name);
    if let Err(e) = create_dir_all(&subdir_path) {
        return Err(format!("Failed to create output subdirectory: {}", e));
    }

    let mut generated_files = Vec::new();

    // Save full visualization
    let dot_path = subdir_path.join(format!("{}.dot", viz_type));
    let svg_path = subdir_path.join(format!("{}.svg", viz_type));
    let png_path = subdir_path.join(format!("{}.png", viz_type));
    let pdf_path = subdir_path.join(format!("{}.pdf", viz_type));

    match fs::write(&dot_path, dot_content) {
        Ok(_) => {
            generated_files.push(dot_path.to_string_lossy().to_string());

            // 1. First generate SVG (which is universally supported)
            match Command::new("dot")
                .args(["-Tsvg", "-o", &svg_path.to_string_lossy()])
                .arg(&dot_path)
                .output()
            {
                Ok(output) => {
                    if output.status.success() && svg_path.exists() {
                        generated_files.push(svg_path.to_string_lossy().to_string());

                        // 2. Then convert SVG to PNG using ImageMagick
                        match Command::new("convert")
                            .arg(&svg_path)
                            .arg(&png_path)
                            .output()
                        {
                            Ok(convert_output) => {
                                if convert_output.status.success() && png_path.exists() {
                                    generated_files.push(png_path.to_string_lossy().to_string());
                                } else if !convert_output.status.success() {
                                    println!(
                                        "Warning: Failed to convert SVG to PNG: {}",
                                        String::from_utf8_lossy(&convert_output.stderr)
                                    );
                                }
                            }
                            Err(e) => {
                                println!(
                                    "Warning: ImageMagick convert not available (install with 'apt-get install imagemagick'): {}",
                                    e
                                );
                            }
                        }
                    } else {
                        return Err(format!(
                            "Failed to generate SVG: {}\n{}",
                            output.status,
                            String::from_utf8_lossy(&output.stderr)
                        ));
                    }
                }
                Err(e) => {
                    return Err(format!(
                        "Failed to execute dot command: {}. \
                        Is GraphViz installed? Try installing with 'brew install graphviz' on macOS or \
                        'apt-get install graphviz' on Linux.",
                        e
                    ));
                }
            }

            // Still generate PDF directly as it doesn't have the same rendering issues as PNG
            match Command::new("dot")
                .args(["-Tpdf", "-o", &pdf_path.to_string_lossy()])
                .arg(&dot_path)
                .output()
            {
                Ok(output) => {
                    if output.status.success() && pdf_path.exists() {
                        generated_files.push(pdf_path.to_string_lossy().to_string());
                    } else if !output.status.success() {
                        println!(
                            "Warning: Failed to generate PDF: {}",
                            String::from_utf8_lossy(&output.stderr)
                        );
                    }
                }
                Err(e) => {
                    println!("Warning: Failed to execute dot for PDF: {}", e);
                }
            }
        }
        Err(e) => return Err(format!("Failed to write DOT file: {}", e)),
    }

    // Try to open the PNG files for viewing (platform-specific)
    if open_files {
        // Try to open the PNG if it exists, otherwise fall back to SVG
        let file_to_open = if png_path.exists() {
            &png_path
        } else if svg_path.exists() {
            &svg_path
        } else {
            println!("Warning: No visualization file exists to open");
            return Ok(generated_files);
        };

        #[cfg(target_os = "macos")]
        match Command::new("open").arg(file_to_open).spawn() {
            Ok(_) => {}
            Err(e) => println!("Warning: Could not open file: {}", e),
        }

        #[cfg(target_os = "linux")]
        match Command::new("xdg-open").arg(file_to_open).spawn() {
            Ok(_) => {}
            Err(e) => println!("Warning: Could not open file: {}", e),
        }

        #[cfg(target_os = "windows")]
        match Command::new("cmd")
            .args(["/C", "start", &file_to_open.to_string_lossy()])
            .spawn()
        {
            Ok(_) => {}
            Err(e) => println!("Warning: Could not open file: {}", e),
        }
    }

    Ok(generated_files)
}
