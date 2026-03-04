use colored::Colorize;

pub fn print_header(text: &str) {
    println!("{}", text.bold().cyan());
}

pub fn print_field(label: &str, value: &str) {
    if !value.is_empty() {
        println!("  {}: {}", label.dimmed(), value);
    }
}

pub fn print_error(msg: &str) {
    eprintln!("{} {}", "error:".red().bold(), msg);
}

pub fn print_success(msg: &str) {
    println!("{} {}", "✓".green().bold(), msg);
}

pub fn format_date(date_str: &str) -> String {
    let format =
        time::format_description::parse_borrowed::<2>("[year]-[month]-[day] [hour]:[minute]");
    match format {
        Ok(fmt) => {
            time::OffsetDateTime::parse(date_str, &time::format_description::well_known::Rfc3339)
                .ok()
                .and_then(|dt| dt.format(&fmt).ok())
                .unwrap_or_else(|| date_str.to_string())
        }
        Err(_) => date_str.to_string(),
    }
}

pub fn print_table(headers: &[&str], rows: &[Vec<String>]) {
    if rows.is_empty() {
        println!("  No results found.");
        return;
    }

    // Calculate column widths
    let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < widths.len() {
                widths[i] = widths[i].max(cell.len());
            }
        }
    }

    // Print header
    let header_line: Vec<String> = headers
        .iter()
        .enumerate()
        .map(|(i, h)| format!("{:width$}", h, width = widths[i]))
        .collect();
    println!("  {}", header_line.join("  ").bold());

    // Print separator
    let sep: Vec<String> = widths.iter().map(|w| "─".repeat(*w)).collect();
    println!("  {}", sep.join("──").dimmed());

    // Print rows
    for row in rows {
        let line: Vec<String> = row
            .iter()
            .enumerate()
            .map(|(i, cell)| {
                let width = widths.get(i).copied().unwrap_or(0);
                format!("{:width$}", cell, width = width)
            })
            .collect();
        println!("  {}", line.join("  "));
    }
}
