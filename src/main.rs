use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use serde::Deserialize;

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
    let data= import("model.bq").unwrap();
    println!("{:#?}", data);
    Ok(())
    
}

#[derive(Deserialize, Debug)]
pub struct AImodel {
    pub name: String,
    pub version: f32,
    pub input_width: u32,
    pub input_height: u32,
    pub description: String,
    pub color_code: String,
    pub task: String,
    pub post_processing: Vec<String>,
    pub classes: Vec<String>,
}

pub fn import(file_path: &str) -> io::Result<(AImodel, Vec<u8>)> {
    // Open the .bq file
    let mut file = File::open(file_path)?;
    let mut file_content = Vec::new();
    file.read_to_end(&mut file_content)?;

    // Validate magic string
    if &file_content[..7] != b"BQMODEL" {
        panic!("Invalid file format: missing BQMODEL magic string");
    }

    // Read version (1 byte)
    let version = file_content[7];
    if version != 1 {
        panic!("Unsupported version: {}", version);
    }

    // Read JSON section length (4 bytes, little-endian)
    let json_length = u32::from_le_bytes(file_content[8..12].try_into().unwrap()) as usize;

    // Extract JSON section
    let json_start = 12;
    let json_end = json_start + json_length;
    let json_data = &file_content[json_start..json_end];
    let json_str = String::from_utf8(json_data.to_vec())
        .unwrap_or_else(|_| panic!("Failed to parse JSON content"));

    // Deserialize JSON into AImodel
    let ai_model: AImodel = serde_json::from_str(&json_str)
        .unwrap_or_else(|_| panic!("Failed to deserialize JSON into AImodel"));

    // Read ONNX section length (4 bytes, little-endian)
    let onnx_length_start = json_end;
    let onnx_length = u32::from_le_bytes(
        file_content[onnx_length_start..onnx_length_start + 4]
            .try_into()
            .unwrap(),
    ) as usize;

    // Extract ONNX section
    let onnx_start = onnx_length_start + 4;
    let onnx_end = onnx_start + onnx_length;
    let onnx_data = file_content[onnx_start..onnx_end].to_vec();

    Ok((ai_model, onnx_data))
}