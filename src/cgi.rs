use std::error::Error;
use std::fs::File;
use std::io::Read;

fn error(code: u32) -> ! {
    println!("Status: {code}");
    println!("Content-Type: text/html");
    println!();
    println!("<h1>{code}</h1>");
    std::process::exit(1)
}

fn unquote(input: &str) -> Result<String, Box<dyn Error>> {
    let mut buf = Vec::with_capacity(input.len());

    let mut in_escape = false;
    let mut high_nibble = None;

    for c in input.bytes() {
        if in_escape && high_nibble.is_none() {
            high_nibble = Some(c)
        } else if in_escape && high_nibble.is_some() {
            let nibbles = String::from_utf8(vec![high_nibble.unwrap(), c])?;
            buf.push(u8::from_str_radix(&nibbles, 16)?);

            high_nibble = None;
            in_escape = false;
        } else if c == b'+' {
            buf.push(b' ');
        } else if c == b'%' {
            in_escape = true;
        } else {
            buf.push(c);
        }
    }

    Ok(String::from_utf8(buf)?.replace("\r\n", "\n"))
}

fn cgi_main() -> Result<(), Box<dyn Error>> {
    if std::env::var("REQUEST_METHOD")? != "POST" {
        Err("Invalid REQUEST_METHOD")?;
    }

    let mut buf = Vec::with_capacity(1024);
    std::io::stdin().read_to_end(&mut buf)?;
    let formdata = String::from_utf8(buf)?;

    let (mut message_text, mut message_id) = (None, None);
    for entry in formdata.split('&') {
        let kv: Vec<&str> = entry.split('=').collect();
        if kv.len() != 2 {
            Err("Invalid formdata entry")?;
        }

        let (key, value) = (kv[0], kv[1]);
        match key {
            "message" => {
                message_text = Some(value);
            }
            "id" => {
                message_id = Some(value);
            }
            _ => {
                Err("Invalid formdata key")?;
            }
        }
    }

    let message_text = super::escape(&unquote(message_text.ok_or("No message form entry")?)?);
    let message_id_str = unquote(message_id.ok_or("No id form entry")?)?;

    if message_id_str == "+" {
        super::add(&message_text)?;
    } else {
        super::edit(message_id_str.parse::<usize>()?, &message_text)?;
    }

    let output = File::options()
        .truncate(true)
        .write(true)
        .open(super::get_render_path())?;
    super::render(output, &super::Entry::read_entries()?)?;

    println!("Status: 302");
    println!("Location: vent.html");
    println!();
    Ok(())
}

pub fn submit_cgi() {
    let res = cgi_main();
    if res.is_err() {
        eprintln!("{res:?}");
        error(400);
    }
}
