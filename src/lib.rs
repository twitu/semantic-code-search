use std::{error::Error, fs};

#[derive(Debug)]
pub struct Coordinates {
    line_number: u32,
    char_start: u32,
    char_end: u32,
}

pub struct Config {
    file_path: String,
    locations: Vec<Coordinates>,
}

impl Config {
    pub fn build(args: &[String]) -> Result<Config, &str> {
        if args.len() < 3 {
            return Err("Too few arguments! Usage: <file_path> <coordinate1> <coordinate2> ...
                        where <coordinate> corresponds to \"line_number,(start_index,end_index)\"\n");
        }

        let file_path = args[1].clone();
        let mut locations: Vec<Coordinates> = vec![];
        for arg in &args[2..] {
            match Config::parse_locations(arg) {
                Ok(coordinate) => locations.push(coordinate),
                Err(e) => eprintln!("Error: {}, Location: {}", e, arg),
            }
        }

        Ok(Config {
            file_path,
            locations,
        })
    }

    fn parse_locations(location: &str) -> Result<Coordinates, &str> {
        let input = location.trim();
        let parts: Vec<&str> = input.split(",(").collect();

        if parts.len() != 2 {
            return Err("Wrong input format. Please use this format : \"line_number,(char_start_index,char_end_index)\"");
        }

        let line_number: u32 = parts[0]
            .trim()
            .parse()
            .map_err(|_| "Invalid line number format")?;
        let char_range = parts[1]
            .strip_suffix(')')
            .ok_or("Missing closing parenthesis in character range")?
            .split(',')
            .map(|s| s.trim().parse::<u32>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| "Invalid character range format")?;

        Ok(Coordinates {
            line_number,
            char_start: char_range[0],
            char_end: char_range[1],
        })
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let contents = match fs::read_to_string(&config.file_path) {
        Ok(contents) => contents,
        Err(e) => {
            return Err(Box::new(e));
        }
    };
    let lines: Vec<&str> = contents.lines().collect();

    for coordinate in config.locations {
        print_location(&coordinate, &lines);
    }

    Ok(())
}

pub fn print_location(coordinate: &Coordinates, lines: &[&str]) {
    println!(
        "Line: {}, Range: [{},{})",
        coordinate.line_number, coordinate.char_start, coordinate.char_end
    );
    if (coordinate.line_number as usize) > lines.len() || coordinate.line_number == 0 {
        println!(
            "Invalid line number: File has only {} lines.\n",
            lines.len()
        );
        return;
    }

    let line = lines[(coordinate.line_number - 1) as usize];
    let line_len = line.len();
    if (coordinate.char_start as usize) >= line_len
        || (coordinate.char_end as usize) > line_len
        || coordinate.char_start >= coordinate.char_end
    {
        println!(
            "Invalid range for line. Line has {} characters.\n",
            line_len
        );
        return;
    }

    println!("{}", line);

    let mut highlight = String::new();
    for i in 0..line.len() {
        if i >= (coordinate.char_start as usize) && i < (coordinate.char_end as usize) {
            highlight.push('^');
        } else {
            highlight.push(' ');
        }
    }
    println!("{}", highlight);
}
