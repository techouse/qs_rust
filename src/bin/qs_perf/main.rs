mod args;
mod cases;
mod measure;
mod payloads;
mod render;

use self::args::{OutputFormat, parse_args};
use self::measure::measure_snapshot;
use self::render::{emit_output, render_snapshot_json, render_snapshot_text};

fn main() {
    let args = parse_args();
    let snapshot = measure_snapshot(&args);
    let rendered = match args.format {
        OutputFormat::Text => render_snapshot_text(&snapshot),
        OutputFormat::Json => render_snapshot_json(&snapshot),
    };
    emit_output(&rendered, args.output.as_deref());
}
