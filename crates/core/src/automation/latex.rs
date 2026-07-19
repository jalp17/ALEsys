use crate::executor::{self, ExecutorConfig};

pub async fn compile_to_pdf(tex_path: &str, output_dir: Option<&str>) -> Result<String, String> {
    let config = ExecutorConfig {
        timeout_ms: 60_000,
        ..Default::default()
    };

    let mut args = vec!["-interaction=nonstopmode", "-halt-on-error"];
    if let Some(dir) = output_dir {
        args.push("-output-directory");
        args.push(dir);
    }
    args.push(tex_path);

    let result = executor::execute("pdflatex", &args, None, &config).await?;

    if result.exit_code != 0 {
        return Err(format!("LaTeX compilation failed:\n{}", result.stderr));
    }

    Ok(result.stdout)
}

pub async fn compile_with_bibtex(tex_path: &str, output_dir: Option<&str>) -> Result<String, String> {
    let config = ExecutorConfig {
        timeout_ms: 120_000,
        ..Default::default()
    };

    let base_path = std::path::Path::new(tex_path);
    let base_name = base_path.file_stem().unwrap_or_default().to_string_lossy();

    compile_to_pdf(tex_path, output_dir).await?;

    let bib_args: Vec<String> = if let Some(dir) = output_dir {
        vec![format!("-output-directory={}", dir), base_name.to_string()]
    } else {
        vec![base_name.to_string()]
    };
    let bib_refs: Vec<&str> = bib_args.iter().map(|s| s.as_str()).collect();
    executor::execute("bibtex", &bib_refs, None, &config).await?;

    compile_to_pdf(tex_path, output_dir).await?;
    compile_to_pdf(tex_path, output_dir).await
}
