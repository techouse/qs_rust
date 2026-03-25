use std::fs;

use super::measure::Snapshot;

pub(super) fn render_snapshot_text(snapshot: &Snapshot) -> String {
    let mut output = String::new();
    output.push_str(&format!(
        "qs_rust perf snapshot (median of {} samples)\n",
        snapshot.samples
    ));

    if !snapshot.encode.is_empty() {
        output.push_str("Encode (encode=false, deep nesting):\n");
        for entry in &snapshot.encode {
            output.push_str(&format!(
                "  depth={:5}: {:8.3} ms/op | len={}\n",
                entry.case.depth, entry.result.ms_per_op, entry.result.output_metric
            ));
        }
    }

    if !snapshot.decode.is_empty() {
        output.push_str("Decode (public API):\n");
        for entry in &snapshot.decode {
            output.push_str(&format!(
                "  {}: count={:4}, comma={:<5}, utf8={:<5}, len={:2}: {:7.3} ms/op | keys={}",
                entry.case.name,
                entry.case.count,
                entry.case.comma,
                entry.case.utf8_sentinel,
                entry.case.value_len,
                entry.result.ms_per_op,
                entry.result.output_metric,
            ));
            output.push('\n');
        }
    }

    output
}

pub(super) fn render_snapshot_json(snapshot: &Snapshot) -> String {
    let mut sections = Vec::new();

    if !snapshot.encode.is_empty() {
        let entries = snapshot
            .encode
            .iter()
            .map(|entry| {
                format!(
                    "{{\"depth\":{},\"iterations\":{},\"ms_per_op\":{:.6},\"length\":{}}}",
                    entry.case.depth,
                    entry.case.iterations,
                    entry.result.ms_per_op,
                    entry.result.output_metric
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        sections.push(format!("\"encode\":[{entries}]"));
    }

    if !snapshot.decode.is_empty() {
        let entries = snapshot
            .decode
            .iter()
            .map(|entry| {
                format!(
                    "{{\"name\":\"{}\",\"count\":{},\"comma\":{},\"utf8\":{},\"value_len\":{},\"iterations\":{},\"ms_per_op\":{:.6},\"keys\":{}}}",
                    entry.case.name,
                    entry.case.count,
                    entry.case.comma,
                    entry.case.utf8_sentinel,
                    entry.case.value_len,
                    entry.case.iterations,
                    entry.result.ms_per_op,
                    entry.result.output_metric
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        sections.push(format!("\"decode\":[{entries}]"));
    }

    format!("{{{}}}", sections.join(","))
}

pub(super) fn emit_output(content: &str, output_path: Option<&str>) {
    if let Some(path) = output_path {
        fs::write(path, content).expect("failed to write perf output");
    } else {
        print!("{content}");
    }
}

#[cfg(test)]
mod tests;
