#[cfg(feature = "vulkan")]
use std::env;
#[cfg(feature = "vulkan")]
use std::fs;
#[cfg(feature = "vulkan")]
use std::path::PathBuf;
#[cfg(feature = "vulkan")]
use std::process::Command;

fn main() {
    // Only compile shaders when vulkan feature is enabled
    #[cfg(feature = "vulkan")]
    {
        compile_vulkan_shaders();
    }
}

#[cfg(feature = "vulkan")]
fn compile_vulkan_shaders() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let shader_dir = PathBuf::from("src/shaders/vulkan");

    // Create output directory for compiled shaders
    let spirv_dir = out_dir.join("spirv");
    fs::create_dir_all(&spirv_dir).expect("Failed to create SPIR-V output directory");

    // List of shaders to compile
    let shaders = [
        "add.comp",
        "mul.comp",
        "sub.comp",
        "div.comp",
        "sin.comp",
        "cos.comp",
        "exp.comp",
        "log.comp",
        "sqrt.comp",
        "pow.comp",
        "matmul.comp",
    ];

    println!("cargo:rerun-if-changed=src/shaders/vulkan");

    for shader in &shaders {
        let shader_path = shader_dir.join(shader);
        let spirv_path = spirv_dir.join(format!("{}.spv", shader));

        println!("cargo:rerun-if-changed={}", shader_path.display());

        // Try to use glslangValidator (from glslang package) or glslc (from shaderc/Vulkan SDK)
        // First try glslangValidator which is available via Homebrew
        let mut result = Command::new("glslangValidator")
            .arg("-V")
            .arg(&shader_path)
            .arg("-o")
            .arg(&spirv_path)
            .status();

        // If glslangValidator not found, try glslc
        if result.is_err() {
            result = Command::new("glslc")
                .arg(&shader_path)
                .arg("-o")
                .arg(&spirv_path)
                .arg("--target-env=vulkan1.2")
                .arg("-O")
                .status();
        }

        match result {
            Ok(status) if status.success() => {
                println!("cargo:warning=Compiled {} to SPIR-V", shader);
            }
            Ok(status) => {
                eprintln!("\n=== Vulkan Shader Compilation Failed ===");
                eprintln!(
                    "Failed to compile shader {}: compiler exited with status {}",
                    shader, status
                );
                eprintln!("\nTo build with Vulkan support, you need a GLSL to SPIR-V compiler.");
                eprintln!("\nInstall via Homebrew:");
                eprintln!("  brew install glslang");
                eprintln!("\nOr download the Vulkan SDK from: https://vulkan.lunarg.com/sdk/home");
                eprintln!("=========================================\n");
                panic!("Shader compilation failed");
            }
            Err(e) => {
                eprintln!("\n=== GLSL Compiler Not Found ===");
                eprintln!("Could not run shader compiler: {}", e);
                eprintln!("\nTo build with Vulkan support, you need a GLSL to SPIR-V compiler.");
                eprintln!("\nInstall via Homebrew:");
                eprintln!("  brew install glslang");
                eprintln!("\nOr download the Vulkan SDK from: https://vulkan.lunarg.com/sdk/home");
                eprintln!("\nAfter installation, make sure 'glslangValidator' or 'glslc' is in your PATH.");
                eprintln!("\nAlternatively, build without Vulkan:");
                eprintln!("  cargo build  (without --features vulkan)");
                eprintln!("================================\n");
                panic!("GLSL compiler not found");
            }
        }
    }

    // Generate Rust code to include the compiled shaders
    let include_file = out_dir.join("shaders.rs");
    let mut include_code = String::new();

    include_code.push_str("// Auto-generated shader includes\n\n");

    for shader in &shaders {
        let shader_name = shader.trim_end_matches(".comp").to_uppercase();
        let spirv_path = spirv_dir.join(format!("{}.spv", shader));

        include_code.push_str(&format!(
            "pub const {}_SPIRV: &[u8] = include_bytes!(\"{}\");\n",
            shader_name,
            spirv_path.display()
        ));
    }

    fs::write(&include_file, include_code).expect("Failed to write shader includes");
}
