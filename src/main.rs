extern crate gtk;
use gtk::prelude::*;
use gtk::{Label, TextView, TextBuffer, TextTagTable, Window, WindowType, Box as GtkBox};
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;
use std::error::Error;
use chrono::{Utc, offset::TimeZone};

fn main() -> Result<(), Box<dyn Error>> {
    // Create the GTK application.
    gtk::init()?;

    // Create the main window.
    let window = Window::new(WindowType::Toplevel);
    window.set_title("Last Epoch Checker");
    window.set_default_size(400, 200);

    // Create a box to hold the label and text view.
    let main_box = GtkBox::new(gtk::Orientation::Vertical, 0);
    window.add(&main_box);

    // Create a label for displaying the last epoch or "error."
    let label = Label::new(None);
    main_box.pack_start(&label, false, false, 0);

    // Create a text view for displaying the last 10 lines when there's an error.
    let text_view_error = TextView::new();
    let text_buffer_error = TextBuffer::new(Some(&TextTagTable::new()));
    text_view_error.set_buffer(Some(&text_buffer_error));
    main_box.pack_start(&text_view_error, true, true, 0);

    // Connect the close event.
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    // Read the last 10 lines from "out.txt" and update the label and text view.
    let (last_epoch, last_lines) = get_last_epoch()?;
    if let Some(epoch) = last_epoch {
        let (normal_date, last_epoch_line) = epoch_to_utc_date_and_last_line(epoch)?;
        label.set_text(&format!("Last Epoch: {} ({})", epoch, normal_date));

        if let Some(line) = last_epoch_line {
            // Add "Validator node running" after a successful epoch detection to the text view.
            text_buffer_error.set_text(&format!("Validator node running\n{}", line));
        } else {
            // If no line is found, just add "Validator node running."
            text_buffer_error.set_text("Validator node running");
        }
    } else {
        label.set_text("Error");

        if let Some(lines) = last_lines {
            text_buffer_error.set_text(&lines);
        } else {
            // Handle the case when the file "out.txt" is not found.
            text_buffer_error.set_text("Error: File 'out.txt' not found.");
        }
    }

    // Show all the widgets.
    window.show_all();

    // Start the GTK main loop.
    gtk::main();

    Ok(())
}

fn get_last_epoch() -> Result<(Option<u64>, Option<String>), Box<dyn Error>> {
    if let Ok(file) = File::open("out.txt") {
        let reader = BufReader::new(file);
        let re = Regex::new(r"\[(\d+)\]").map_err(|e| e.to_string())?;
        let mut last_lines: Vec<String> = Vec::new();

        for line in reader.lines().filter_map(Result::ok) {
            last_lines.push(line.clone());

            if last_lines.len() > 10 {
                last_lines.remove(0);
            }
        }
        // Attempt to find the last epoch after reading all lines.
        let last_epoch = last_lines
            .iter()
            .rev()
            .find_map(|line| {
                if let Some(captures) = re.captures(&line) {
                    if let Some(epoch_str) = captures.get(1) {
                        epoch_str.as_str().parse::<u64>().ok()
                    } else {
                        None
                    }
                } else {
                    None
                }
            });
        Ok((last_epoch, if last_epoch.is_none() { Some(last_lines.join("\n")) } else { None }))
    } else {
        Ok((None, None))
    }
}

fn epoch_to_utc_date_and_last_line(epoch: u64) -> Result<(String, Option<String>), Box<dyn Error>> {
    let datetime = Utc.timestamp(epoch as i64, 0);
    let normal_date = datetime.format("%Y-%m-%d %H:%M:%S").to_string();
    Ok((normal_date, None))
}
