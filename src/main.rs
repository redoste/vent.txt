use std::env;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufReader, Error as IoError, ErrorKind};

use chrono::prelude::*;

use handlebars as hb;
use handlebars::{Handlebars, RenderError, Renderable};
use serde::Serialize;

fn get_csv_path() -> String {
    env::var("VENT_TXT_CSV").unwrap_or_else(|_| String::from("vent.csv"))
}

fn get_template_path() -> String {
    env::var("VENT_TXT_HBS").unwrap_or_else(|_| String::from("template/vent.hbs"))
}

fn collect_message_from_args(args: env::Args) -> Result<String, IoError> {
    let message = args.collect::<Vec<String>>().join(" ").trim().to_owned();
    if message.is_empty() {
        Err(IoError::new(ErrorKind::InvalidInput, "Empty message"))
    } else if message.contains('\n') || message.contains('\r') {
        Err(IoError::new(
            ErrorKind::InvalidInput,
            "Message contains new line",
        ))
    } else {
        Ok(message)
    }
}

fn collect_message_id_from_args(args: &mut env::Args) -> Result<usize, IoError> {
    args.next()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| IoError::new(ErrorKind::InvalidInput, "Invalid message ID"))
}

#[derive(Serialize)]
struct Entry {
    date: String,
    reply: Option<usize>,
    message: String,
}

impl Entry {
    fn read_raw_entries() -> Result<Vec<String>, IoError> {
        BufReader::new(File::open(get_csv_path())?)
            .lines()
            .collect()
    }

    fn read_entries() -> Result<Vec<Self>, IoError> {
        let raw_entries = Self::read_raw_entries()?;
        raw_entries.iter().map(|s| Self::parse_entry(s)).collect()
    }

    fn parse_entry(raw_entry: &str) -> Result<Self, IoError> {
        let date_end = raw_entry
            .find(',')
            .ok_or_else(|| IoError::new(ErrorKind::InvalidData, "No date in entry"))?;

        let (date, message) = raw_entry.split_at(date_end);
        let message = &message[1..]; // We drop the separating comma

        let (reply, message) =
            if message.len() > 2 && message.is_char_boundary(2) && &message[..2] == ">>" {
                let reply_end = message.find(' ').unwrap_or(message.len());
                let reply_text = &message[2..reply_end];
                let reply = reply_text.parse().ok();
                let message_start = if reply.is_some() { reply_end } else { 0 };
                (reply, &message[message_start..])
            } else {
                (None, message)
            };

        Ok(Entry {
            date: date.to_owned(),
            reply,
            message: message.to_owned(),
        })
    }
}

fn format_local_time() -> String {
    Local::now().format("%Y-%m-%d %H:%M:%S %z").to_string()
}

fn add(message: &str) -> Result<(), IoError> {
    let mut file = File::options()
        .create(true)
        .append(true)
        .open(get_csv_path())?;
    let date = format_local_time();
    writeln!(file, "{date},{message}")?;
    Ok(())
}

fn edit(message_id: usize, message: &str) -> Result<(), IoError> {
    let mut entries = Entry::read_raw_entries()?;
    let date = format_local_time();
    match entries.get_mut(message_id) {
        Some(s) => *s = format!("{date},{message}"),
        None => {
            return Err(IoError::new(
                ErrorKind::InvalidInput,
                "Out-of-bound message ID",
            ))
        }
    }

    let mut file = File::options().write(true).open(get_csv_path())?;
    for entry in entries.iter() {
        writeln!(file, "{entry}")?;
    }
    Ok(())
}

struct RenderIfReplyHelper;

/* We don't use the built-in `if` helper as it follows javascript logic and will treat 0 as false.
 * This helper will treat null as false and any number as true.
 */
impl hb::HelperDef for RenderIfReplyHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        helper: &hb::Helper<'reg, 'rc>,
        registry: &'reg Handlebars<'reg>,
        context: &'rc hb::Context,
        render_context: &mut hb::RenderContext<'reg, 'rc>,
        out: &mut dyn hb::Output,
    ) -> Result<(), RenderError> {
        let value = helper
            .param(0)
            .ok_or_else(|| RenderError::new("Param not found for helper \"if_reply\""))?
            .value();
        if value.is_number() {
            let template = helper
                .template()
                .ok_or_else(|| RenderError::new("Template not found for helper \"if_reply\""))?;
            template.render(registry, context, render_context, out)
        } else if value.is_null() {
            Ok(())
        } else {
            Err(RenderError::new(
                "Param of invalid type for helper \"if_reply\"",
            ))
        }
    }
}

struct RenderEachReverseHelper;

/* We don't use the built-in `for_each` helper as we can't easily use to it to iterate backwards
 * while keeping the original indices in the `index` local variable.
 */
impl hb::HelperDef for RenderEachReverseHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        helper: &hb::Helper<'reg, 'rc>,
        registry: &'reg Handlebars<'reg>,
        context: &'rc hb::Context,
        render_context: &mut hb::RenderContext<'reg, 'rc>,
        out: &mut dyn hb::Output,
    ) -> Result<(), RenderError> {
        let param = helper
            .param(0)
            .ok_or_else(|| RenderError::new("Param not found for helper \"each_reverse\""))?
            .value();
        let template = helper
            .template()
            .ok_or_else(|| RenderError::new("Template not found for helper \"each_reverse\""))?;
        if param.is_array() {
            for (index, value) in param.as_array().unwrap().iter().enumerate().rev() {
                if let Some(block) = render_context.block_mut() {
                    block.set_local_var("index", serde_json::json!(index));
                    block.set_base_value(value.clone());
                }
                template.render(registry, context, render_context, out)?
            }
            Ok(())
        } else {
            Err(RenderError::new(
                "Param of invalid type for helper \"each_reverse\"",
            ))
        }
    }
}

fn render<W>(writer: W, entries: &Vec<Entry>) -> Result<(), RenderError>
where
    W: io::Write,
{
    let template_name = "template";
    let mut handlebars = Handlebars::new();
    handlebars.register_template_file(template_name, get_template_path())?;
    handlebars.register_helper("if_reply", Box::new(RenderIfReplyHelper));
    handlebars.register_helper("each_reverse", Box::new(RenderEachReverseHelper));
    handlebars.render_to_write(template_name, &entries, writer)
}

fn usage(program_name: &str) -> ! {
    eprintln!("Usage: {program_name} add [message]");
    eprintln!("       {program_name} add '>>[reply id]' [message]");
    eprintln!("       {program_name} edit [message id] [message]");
    eprintln!("       {program_name} rm [message id]");
    eprintln!("       {program_name} render");
    eprintln!();
    eprintln!("Environment: VENT_TXT_CSV    Vent database location");
    eprintln!("                             (default: 'vent.csv')");
    eprintln!("             VENT_TXT_HBS    Render template");
    eprintln!("                             (default: 'template/vent.hbs')");
    std::process::exit(1)
}

fn main() -> Result<(), IoError> {
    let mut args = env::args();
    let program_name = args.next().unwrap();
    let action = args.next().unwrap_or_else(|| usage(&program_name));

    match action.as_str() {
        "add" => add(collect_message_from_args(args)?.as_str()),
        "edit" => {
            let message_id = collect_message_id_from_args(&mut args)?;
            let message = collect_message_from_args(args)?;
            edit(message_id, &message)
        }
        "rm" => {
            let message_id = collect_message_id_from_args(&mut args)?;
            edit(message_id, "[removed]")
        }
        "render" => match render(io::stdout(), &Entry::read_entries()?) {
            Ok(_) => Ok(()),
            Err(e) => {
                eprintln!("{e}");
                if let Some(es) = e.source() {
                    eprintln!("{:?}", es);
                }
                Err(IoError::new(ErrorKind::Other, "Render error"))
            }
        },
        _ => usage(&program_name),
    }
}
