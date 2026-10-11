use ironjump::{game::Game, levels::Level};
use std::path::{Path, PathBuf};

fn custom_level(path: &Path) -> Result<Level, String> {
    let xml =
        std::fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
    Game::new()
        .load(&xml)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    Ok(Level {
        name: path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned(),
        xml,
        custom: true,
    })
}

pub fn options() -> Result<(Vec<Level>, bool, PathBuf), String> {
    let mut additional = Vec::new();
    let mut start = false;
    let mut progress = PathBuf::from("ironjump-progress.txt");
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--level" => {
                let path = args.next().ok_or("--level requires an .xlevel path")?;
                additional.push(custom_level(Path::new(&path))?);
                start = true;
            }
            "--levels-dir" => {
                let path = args.next().ok_or("--levels-dir requires a directory")?;
                let mut files = std::fs::read_dir(path)
                    .map_err(|error| error.to_string())?
                    .map(|entry| entry.map(|entry| entry.path()))
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|error| error.to_string())?;
                files.sort();
                for file in files {
                    if file
                        .extension()
                        .is_some_and(|extension| extension == "xlevel")
                    {
                        additional.push(custom_level(&file)?);
                    }
                }
            }
            "--progress" => {
                progress = PathBuf::from(args.next().ok_or("--progress requires a file path")?)
            }
            _ => {
                return Err(format!(
                    "Unknown argument {arg}. Use --level FILE, --levels-dir DIR, or --progress FILE."
                ));
            }
        }
    }
    Ok((additional, start, progress))
}
