use std::env;
use std::fs::File;
use std::io::{self, Read, Write};

fn main() -> io::Result<()> {
    // Get command-line arguments
    let args: Vec<String> = env::args().collect();

    // Variables to hold file paths
    let (json_path, onnx_path, output_path) = if args.len() == 2 {
        // Single parameter mode
        let base_name = &args[1];
        (
            format!("{}.json", base_name),
            format!("{}.onnx", base_name),
            format!("{}.bq", base_name),
        )
    } else if args.len() == 4 {
        // Three-parameter mode
        (args[1].clone(), args[2].clone(), args[3].clone())
    } else {
        eprintln!(
            "Usage:\n  {} <base_name>\n  {} <json_path> <onnx_path> <output_path>",
            args[0], args[0]
        );
        std::process::exit(1);
    };

    // Read the JSON file
    let mut json_file = File::open(&json_path)
        .unwrap_or_else(|_| panic!("Failed to open JSON file: {}", json_path));
    let mut json_content = Vec::new();
    json_file.read_to_end(&mut json_content)?;

    // Read the ONNX file
    let mut onnx_file = File::open(&onnx_path)
        .unwrap_or_else(|_| panic!("Failed to open ONNX file: {}", onnx_path));
    let mut onnx_content = Vec::new();
    onnx_file.read_to_end(&mut onnx_content)?;

    // Create the .bq file
    let mut output_file = File::create(&output_path)
        .unwrap_or_else(|_| panic!("Failed to create output file: {}", output_path));

    // Write header
    output_file.write_all(b"BQMODEL")?; // Magic string
    output_file.write_all(&[1])?; // Version

    // Write JSON section
    let json_length = json_content.len() as u32;
    output_file.write_all(&json_length.to_le_bytes())?; // Length of JSON
    output_file.write_all(&json_content)?; // JSON content

    // Write ONNX section
    let onnx_length = onnx_content.len() as u32;
    output_file.write_all(&onnx_length.to_le_bytes())?; // Length of ONNX
    output_file.write_all(&onnx_content)?; // ONNX content

    println!("Combined file written to {}", output_path);

    Ok(())
}
