use std::fs;
use std::path::Path;
use std::process::Command;

#[test]
fn test_successful_compilation(){
    let input_folder = Path::new("tests/input/");
    let test_file = input_folder.join("test1_input.brs");
    let output_folder = Path::new("tests/output/");
    let executable = {
        #[allow(unused_mut)] // The variable will be only modified on Windows
        let mut exec_file_path = output_folder.join("test1_input");
        // On Windows, add the .exe extension.
        #[cfg(windows)]
        {
            exec_file_path.set_extension("exe");
        }
        #[cfg(unix)]
        {
            exec_file_path = Path::new("./").join(exec_file_path)
        }
        exec_file_path
    };
    

    fs::remove_dir_all(input_folder).ok();
    fs::remove_dir_all(output_folder).ok();
    fs::create_dir_all(input_folder).expect("Failed to create input folder");
    fs::create_dir_all(output_folder).expect("Failed to create output folder");
    
    let source_code = r#"
    x = ((3+5)*2 + (12//4))%7+(18//(6-3))*(2**3-4) + 10
    y = true
    z = true && false
    {
        x = 0
        exit(x)
    }
    exit(x)
    "#;
    fs::write(&test_file, source_code).expect("Unable to write file");
    
    let output = Command::new("cargo")
        .args(["run", test_file.to_str().unwrap(), "--outdir", output_folder.to_str().unwrap()])
        .output()
        .expect("Failed to run compiler");

    assert!(
        output.status.success(),
        "Compiler failed with stderr: {}\nThe Compiler stdout was: {}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );

    // Run the compiled executable
    #[cfg(unix)]
    {
        use std::fs::Permissions;
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&executable, Permissions::from_mode(0o755))
            .expect("Failed to set execute permissions on the binary");
    }
    let run_output = if cfg!(windows) {
        Command::new("cmd")
            .args(executable.canonicalize())
            .output()
            .expect("Failed to execute compiled binary")
    } else {
        Command::new(&executable)
            .output()
            .expect("Failed to execute compiled binary")
    };

    // Ensure execution was successful
    assert!(
        run_output.status.success(),
        "Execution Command {:?} failed.\n Execution failed with stderr: {}\nThe program stdout was: {}",
        run_output,
        String::from_utf8_lossy(&run_output.stderr),
        String::from_utf8_lossy(&run_output.stdout)
    );

    fs::remove_dir_all(input_folder).unwrap();
    fs::remove_dir_all(output_folder).unwrap()
}