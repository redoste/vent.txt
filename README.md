# vent.txt

**vent.txt** is a simple command line tool to add short, single-line messages to a database and render them as an HTML document. You can see this as some kind of single-user microblogging service.

While it can be adapted for other uses, it was designed to be used to implement *SVoH ("Shouting into the Void over HTTP")* and as a mental health journal.

## Build

```console
$ cargo build --release
$ ./target/release/vent
```

## Usage

```console
$ # Add message "hello"
$ vent add hello
$ # Reply to message 10 with "hello"
$ vent add '>>10' hello
$ # Edit message 15 to "hi"
$ vent edit 15 hi
$ # Remove message 15
$ vent rm 15
$ # Render to static/vent.html
$ vent render > static/vent.html
```

Two environment variables are used to configure the location of important files
* `VENT_TXT_CSV` : Database (default: `./vent.csv`)
* `VENT_TXT_HBS` : Template (default: `./template/vent.hbs`)

## Customization

The provided files were designed for my use, you will probably want to edit them to fit your situation
* `static/index.css` : theme
* `static/index.html` : home page with a content warning
* `template/vent.hbs` : template with a paragraph explaining the concept
