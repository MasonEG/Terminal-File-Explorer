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


fn find_file_name(p: PathBuf) -> String {
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


fn update_dir(path: PathBuf) -> Result<DirBuffer, Error> {
    let dir_contents: Vec<PathBuf> = read_dir(path.clone())?
        .map(|p| p.unwrap().path())
        .collect(); 
    
    
    Ok(DirBuffer {
        path: path.clone(),
        dirs: dir_contents.clone().into_iter().filter_map(|p| {
                if p.is_dir() {
                    return Some(Dir {
                        path: p.clone(),
                        name: find_file_name(p.clone())
                        });
                }
                None
            })
            .collect(),
        files: dir_contents.into_iter().filter_map(|p| {
                if p.is_file() {
                    return Some(find_file_name(p))
                }
                None
            })    
            .collect()
    })
}

// TODO: get this to work
fn set_current_dir(p: PathBuf) {
    let path_str = p.to_str().expect("Failed to convert stored path into string :^|");

    env::set_var("PWD", path_str);
}

fn main() -> Result<(), Error> {
    let dir_color = &color::Fg(color::Blue);
    let file_color = &color::Fg(color::Green);
    let select_color = &color::Bg(color::Rgb(255, 153, 0)); // orange

    // setup vars for the main loop
    let mut stdout = stdout().into_raw_mode().unwrap();
    let stdin = std::io::stdin();
    let mut input = &mut stdin.keys();
    let current_dir: PathBuf = env::current_dir()?;
    let mut dir_contents: &mut DirBuffer = &mut update_dir(current_dir).unwrap();
    let mut dir_index = &mut 0;

    print!("{}", cursor::Hide);

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

        //print the directory
        let term_dimensions = termion::terminal_size().expect("Failed to retrieve size of terminal");
        let filepath_str: String = String::from(dir_contents.path.clone().to_str().unwrap());
        print!("{}{}", color::Fg(color::Red), color::Bg(color::Green));
        print!("{}{}", cursor::Goto(1, term_dimensions.1), filepath_str);
        stdout.flush().unwrap();

        //reset colorscheme
        print!("{}{}", color::Fg(color::Reset), color::Bg(color::Reset));
        
        // handle keyboard events
        let c = input.next();
        match c.unwrap().unwrap() {
            Key::Char('q') => {
                env::set_current_dir(dir_contents.path.as_path()).is_ok();
                set_current_dir(dir_contents.path.clone());
                break;
            },
            Key::Char('j') => {
                let dir_count = dir_contents.dirs.len();
                if dir_count > 0 && *dir_index < (dir_count  - 1) {
                    *dir_index += 1;
                }
            },
            Key::Char('k') => {
                if *dir_index > 0 {
                    *dir_index -= 1;
                }
            },
            Key::Char('h') => {
                let mut path = dir_contents.path.clone();
                path.pop();
                *dir_contents = update_dir(path).unwrap();
                *dir_index = 0;
            },
            Key::Char('l') => {
                if dir_contents.dirs.len() > 0 {
                    let path = dir_contents.dirs.get(*dir_index).unwrap().path.clone();
                    *dir_contents = update_dir(path).unwrap();
                    *dir_index = 0;
                }
            },
            _ => {}
        }
        stdout.flush().unwrap();
    }

    print!("\r{}", termion::clear::All);
    // TODO: reset the cursor

    Ok(())
}
