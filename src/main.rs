use termion::event::Key;
use termion::input::TermRead;
use termion::color;
use termion::raw::IntoRawMode;
use termion::cursor;
use std::io::{Write, stdout};
use std::fs::{ReadDir, DirEntry, read_dir};
use std::path::PathBuf;
use std::env;
use std::io::Error;
use std::vec;
use std::ffi::OsStr;


#[derive(Clone)]
struct Dir {
    path: PathBuf,
    name: String,
}

#[derive(Clone)]
struct DirBuffer {
    path: PathBuf,
    dirs: Vec<Dir>,
    files: Vec<String>,
}


fn findFileNameFromPathBuf(p: PathBuf) -> String {
    let os_str: &OsStr = match p.file_name() {
        Some(s) => s,
        None => panic!(format!("Failed to grab OS string from path: {:?}", p)),
    };

    let s = match os_str.to_str() {
       Some(s) => s,
       None => panic!(format!("Failed to convert OsStr to str: {:?}", os_str)), 
    };

    String::from(s)
}


fn updateDir(path: PathBuf) -> Result<DirBuffer, Error> {
    let dir_contents: Vec<PathBuf> = read_dir(path.clone())?
        .map(|p| p.unwrap().path())
        .collect(); 
    
    
    Ok(DirBuffer {
        path: path.clone(),
        dirs: dir_contents.clone().into_iter().filter_map(|p| {
                if p.is_dir() {
                    return Some(Dir {
                        path: p.clone(),
                        name: findFileNameFromPathBuf(p.clone())
                        });
                }
                None
            })
            .collect(),
        files: dir_contents.into_iter().filter_map(|p| {
                if p.is_file() {
                    return Some(findFileNameFromPathBuf(p))
                }
                None
            })    
            .collect()
        })
    }

fn main() -> Result<(), Error> {
    let current_dir: PathBuf = env::current_dir()?;
    let mut dir_contents: &mut DirBuffer = &mut updateDir(current_dir).unwrap();
    let dir_color = &color::Fg(color::Blue);
    let file_color = &color::Fg(color::Green);
    let select_color = &color::Bg(color::Rgb(255, 153, 0)); // orange
    let stdin = std::io::stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();
    let mut dir_index = &mut 0;
    let mut input = &mut stdin.keys();

    write!(stdout,
           "{}{}q to exit. Type stuff, use alt, and so on.{}",
           termion::clear::All,
           cursor::Goto(1, 1),
           cursor::Hide)
           .unwrap();
    stdout.flush().unwrap();

    loop {
        // clear out the screen
        write!(stdout,
               "{}{}",
               cursor::Goto(1, 1),
               termion::clear::CurrentLine)
                .unwrap();
        print!("{}", termion::clear::All);

        // print the dirs
        print!("{}", dir_color);
        let mut i = &mut 0;
        for d in &dir_contents.dirs {
            if i == dir_index {
                println!("\r{}/{}/", select_color, d.name);             
                print!("{}", color::Bg(color::Reset));
            } else {
                println!("\r/{}/", d.name);             
            }
            
            *i += 1;
        }

        // print the files
        print!("{}", file_color);
        for f in &dir_contents.files {
            println!("\r{}", f);
        }
        
        // handle keyboard events
        let c = input.next();
        match c.unwrap().unwrap() {
            Key::Char('q') => break,
            Key::Char('j') => {
                if *dir_index < (dir_contents.dirs.len()  - 1) {
                    *dir_index += 1;
                }
            },
            Key::Char('k') => {
                if *dir_index > 0 {
                    *dir_index -= 1;
                }
            },
            Key::Char('h') => {
                let mut path = env::current_dir().unwrap();
                path.pop();
                *dir_contents = updateDir(path).unwrap();
                *dir_index = 0;
            },
            Key::Char('l') => {
                let path = dir_contents.dirs.get(*dir_index).unwrap().path.clone();
                *dir_contents = updateDir(path).unwrap();
                *dir_index = 0;
            },
            Key::Char(c) => println!("{}", c),
            Key::Alt(c) => println!("^{}", c),
            Key::Ctrl(c) => println!("*{}", c),
            _ => {}
        }
        stdout.flush().unwrap();
    }
    Ok(())
}
