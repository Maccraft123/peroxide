mod entry;
mod acetone;
mod bootloaderspec;
mod uefi;

use crate::entry::BootEntry;

use anyhow::Result;
use anyhow::Context;

use crossterm::{
    ExecutableCommand,
    QueueableCommand,
    cursor,
    terminal,
    style,
    event,
    event::{
        Event,
        KeyCode,
    }
};

use ez_input::RinputerHandle;
use ez_input::EzEvent;
use std::sync::mpsc::Sender;
use std::sync::mpsc::channel;
use std::io::stdout;
use std::io::Stdout;
use std::io::Write;
use std::rc::Rc;

fn kbd_input(tx: Sender<EzEvent>) {
    loop {
        match event::read().unwrap() {
            Event::Key(key) => {
                match key.code {
                    KeyCode::Enter  => tx.send(EzEvent::South(true)).unwrap(),
                    KeyCode::Down   => tx.send(EzEvent::DirectionDown).unwrap(),
                    KeyCode::Up     => tx.send(EzEvent::DirectionUp).unwrap(),
                    _ => (),
                }
            },
            _ => (),
        }
    }
}

fn pad_input(tx: Sender<EzEvent>) {
    if let Some(mut handle) = RinputerHandle::open() {
        loop {
            let ev = handle.get_event_blocking().unwrap();
            tx.send(ev).unwrap();
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum MenuType {
    Default,
    BootMenu,
    ShowAll,
}

#[derive(Clone, Debug, PartialEq)]
enum MenuChoice {
    Entry(String),
    Submenu(&'static str, Vec<MenuChoice>),
    PowerOff,
}

fn draw_menu(choices: &Vec<MenuChoice>, out: &mut Stdout, cursor_pos: u16) -> Result<()> {
    let origin_x = 4;
    let origin_y = 3;

    // clear terminal and draw title
    out.queue(terminal::Clear(terminal::ClearType::All))?;
    out.queue(cursor::MoveTo(1, 1))?;
    out.queue(style::Print("Choose boot selection"))?;

    // draw all entries
    out.queue(cursor::MoveTo(origin_x, origin_y))?;
    for c in choices {
        match c {
            MenuChoice::Entry(title) => out.queue(style::Print(title.to_string()))?,
            MenuChoice::Submenu(title, _) => out.queue(style::Print(title.to_string()))?,
            MenuChoice::PowerOff => out.queue(style::Print("Shut down"))?,
        };
        out.queue(cursor::MoveToNextLine(1))?;
        out.queue(cursor::MoveToColumn(4))?;
    }

    // draw the cursor
    let cursor_y = origin_y + cursor_pos;
    out.queue(cursor::MoveTo(1, cursor_y))?;
    out.queue(style::Print("=>"))?;

    // and draw, this time for real
    out.flush()?;

    Ok(())
}

#[derive(Debug, PartialEq, Copy, Clone)]
struct PowerOffEntry();
impl BootEntry for PowerOffEntry {
    fn user_readable_name(&self) -> &str {
        "Shut Down"
    }
    fn enumerate() -> Vec<Self> {
        vec![Self()]
    }
    fn boot(&self) {
        todo!("Powering off");
    }
    fn hide(&self) -> bool { false }
}

fn bootentry_pwroff() -> Box<dyn BootEntry> {
    Box::new(PowerOffEntry())
}

fn menu(choices: &Vec<Box<dyn BootEntry>>) -> Result<&Box<dyn BootEntry>> {
    let mut out = stdout();

    let (tx1, rx) = channel::<EzEvent>();
    let tx2 = tx1.clone();
    std::thread::spawn(move || kbd_input(tx1));
    std::thread::spawn(move || pad_input(tx2));

    let mut items_main = Vec::new();
    let mut entries_main = Vec::new();
    let mut items_boot = Vec::new();
    let mut entries_boot = Vec::new();

    for entry in choices.into_iter() {
        if !entry.hide() {
            items_main.push(MenuChoice::Entry(entry.user_readable_name().to_string()));
            entries_main.push(entry);
        } else {
            items_boot.push(MenuChoice::Entry(entry.user_readable_name().to_string()));
            entries_boot.push(entry);
        }
    }

    //items_boot.push(MenuChoice::PowerOff);
    items_main.push(MenuChoice::Submenu("Launch full boot menu", items_boot));

    // prepare for neat display
    out.execute(terminal::EnterAlternateScreen)?;
    out.execute(cursor::Hide)?;
    terminal::enable_raw_mode()?;

    let mut cur_menu = &items_main;
    let mut cur_entries = &mut entries_main;

    let mut pos: i32 = 0;
    let mut quit = false;

    while !quit {
        draw_menu(cur_menu, &mut out, pos.try_into().unwrap_or_default())?;

        match rx.recv()? {
            EzEvent::DirectionDown => pos += 1,
            EzEvent::DirectionUp => pos -= 1,
            EzEvent::South(val) => {
                if val == true {
                    if let Some(c) = cur_menu.get(pos as usize) {
                        if let MenuChoice::Submenu(_, new) = c {
                            if new.len() > 0 {
                                cur_menu = new;
                                // TODO: find a better way
                                cur_entries = &mut entries_boot;
                            }
                        } else {
                            quit = true;
                        }
                    }
                }
            },
            _ => (),
        }

        if pos < 0 || cur_menu.len() == 0 {
            pos = 0;
        }
        if pos as usize + 1 > cur_menu.len() {
            pos = cur_menu.len().try_into().unwrap_or_default();
            pos -= 1;
        }
    }

    // clean up after ourselves
    out.execute(cursor::Show)?;
    out.execute(terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    let ret = cur_entries.swap_remove(pos as usize);
    Ok(ret)
}

fn main() {
    let mut entries = entry::enumerate_all();
    entries.push(bootentry_pwroff());
    let choice = menu(&entries).unwrap();
    println!("Chosen boot option: {}", choice.user_readable_name());
    choice.boot();
}
