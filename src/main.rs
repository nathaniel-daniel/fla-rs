use fla::Fla;
use std::{
    fs::File,
    path::{
        Path,
        PathBuf,
    },
    time::Instant,
};

#[derive(argh::FromArgs, Debug)]
#[argh(description = "A tool to test an fla parser")]
struct FlaCmd {
    #[argh(option)]
    #[argh(description = "the path to an fla file")]
    fla_path: PathBuf,

    #[argh(option)]
    #[argh(description = "the symbol to render")]
    symbol: String,

    #[argh(option)]
    #[argh(description = "the scale of the symbol")]
    scale: Option<f64>,

    #[argh(option)]
    #[argh(description = "the padding of the symbol")]
    padding: Option<f64>,
}

fn main() {
    let fla_cmd: FlaCmd = argh::from_env();

    println!("Opening fla file '{}'", fla_cmd.fla_path.display());
    println!();

    let fla_file = match File::open(&fla_cmd.fla_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to open file: {}", e);
            return;
        }
    };

    let parsed_fla_file = match Fla::new(fla_file) {
        Ok(fla) => fla,
        Err(e) => {
            eprintln!("Failed to parse fla: {}", e);
            return;
        }
    };

    let symbol = match parsed_fla_file
        .get_library_asset(&fla_cmd.symbol)
        .and_then(|s| s.as_xml())
    {
        Some(s) => s,
        None => {
            eprintln!("Could not locate symbol '{}'", fla_cmd.symbol);
            return;
        }
    };

    let scale = fla_cmd.scale.unwrap_or(1.0);
    let padding = fla_cmd.padding.unwrap_or(20.0 * scale);

    if let Some(bounding_box) = symbol.calc_bounding_box() {
        println!("Symbol Bounding Box");
        println!("  Start: {} x {}", bounding_box.min.x, bounding_box.min.y);
        println!("  End: {} x {}", bounding_box.max.x, bounding_box.max.y);
        println!();
    }

    println!("Using scale: {}x", scale);
    println!("Using padding: {}px", padding);
    println!();

    println!("Beginning render...");
    let render_start = Instant::now();

    let frames = match symbol.render_raqote(scale, padding) {
        Ok(frames) => frames,
        Err(e) => {
            eprintln!("Failed to render: {}", e);
            return;
        }
    };

    let render_finish = Instant::now();
    println!("Num frames: {}", frames.len());
    println!(
        "Finished render in {} secs",
        (render_finish - render_start).as_secs_f32()
    );
    println!();

    let mut export_path = Path::new(&fla_cmd.symbol).with_extension("");
    println!("Saving frames in '{}'", export_path.display());

    if let Err(e) = std::fs::create_dir_all(&export_path) {
        eprintln!("Failed to create export directory: {}", e);
        return;
    }

    for (i, frame) in frames.iter().enumerate() {
        export_path.push(format!("{}.png", i));

        if let Err(e) = frame.write_png(&export_path) {
            eprintln!("Failed to write '{}': {}", export_path.display(), e);
        }

        export_path.pop();
    }

    println!("Done.");
}
