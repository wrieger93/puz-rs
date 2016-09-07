#[macro_use]
extern crate nom;

use std::io::prelude::*;
use std::fs::File;

use nom::{le_u8, le_u16, Err, ErrorKind, IResult};

const MAGIC_STRING: &'static str = "ACROSS&DOWN\0";

#[derive(Clone, Debug)]
struct PuzFile<'a> {
    // extra bytes at the beginning
    pre_bytes: &'a [u8],

    // file header
    file_checksum: u16,
    base_checksum: u16,
    masked_low_checksums: &'a [u8],
    masked_high_checksums: &'a [u8],
    version: String,
    reserved_1c: &'a [u8],
    scrambled_checksum: u16,
    reserved_20: &'a [u8],
    width: u8,
    height: u8,
    num_clues: u16,
    unknown_bitmask: u16,
    scrambled_tag: u16,

    // file body
    solution: Vec<char>,
    grid: Vec<char>,
    title: String,
    author: String,
    copyright: String,
    clues: Vec<String>,
    notes: String,

    // extra bytes at the end
    post_bytes: &'a [u8],
}

named!(nul_terminated_string<&[u8], String>,
       map!(
           terminated!(take_until!("\0"), tag!("\0")),
           |bytes: &[u8]| -> String {
               bytes.iter().map(|&x| x as char).collect()
           }
           )
       );

fn pre_bytes_parser(bytes: &[u8]) -> IResult<&[u8], &[u8]> {
    // look for the magic string
    let mut index = 0;
    let mut magic_string_found = false;
    for window in bytes.windows(MAGIC_STRING.len()) {
        if window == MAGIC_STRING.as_bytes() {
            magic_string_found = true;
            break;
        }
        index += 1;
    }

    // there needs to be a two byte checksum before the magic string
    if index < 2 {
        return IResult::Error(Err::Code(ErrorKind::Custom(1)));
    }

    // return an error if the magic string wasn't found
    if !magic_string_found {
        return IResult::Error(Err::Code(ErrorKind::Custom(2)));
    }

    let starts_at = index - 2;
    IResult::Done(&bytes[starts_at..], &bytes[..starts_at])
}

named!(version_parser<&[u8], String>,
        map!(
            take!(4),
            |bytes: &[u8]| -> String {
                bytes[..(bytes.len()-1)].iter().map(|&x| x as char).collect()
            }
        )
);

fn grid_parser(bytes: &[u8], size: usize) -> IResult<&[u8], Vec<char>> {
    map!(
        bytes,
        take!(size),
        |bytes: &[u8]| -> Vec<char> {
            bytes.iter().map(|&x| x as char).collect()
        }
    )
}

fn post_bytes_parser(bytes: &[u8]) -> IResult<&[u8], &[u8]> {
    // just consume all the bytes
    IResult::Done(&[], bytes)
}


named!(full<&[u8], PuzFile>,
       chain!(
           // extra bytes at the beginning
           pre_bytes: pre_bytes_parser ~

           // file header
           file_checksum: le_u16 ~
           tag!(MAGIC_STRING) ~
           base_checksum: le_u16 ~
           masked_low_checksums: take!(4) ~
           masked_high_checksums: take!(4) ~
           version: version_parser ~
           reserved_1c: take!(2) ~
           scrambled_checksum: le_u16 ~
           reserved_20: take!(12) ~
           width: le_u8 ~
           height: le_u8 ~
           num_clues: le_u16 ~
           unknown_bitmask: le_u16 ~
           scrambled_tag: le_u16 ~

           // file body
           solution: apply!(grid_parser, (width as usize) * (height as usize)) ~
           grid: apply!(grid_parser, (width as usize) * (height as usize)) ~
           title: nul_terminated_string ~
           author: nul_terminated_string ~
           copyright: nul_terminated_string ~
           clues: count!(nul_terminated_string, num_clues as usize) ~
           notes: nul_terminated_string ~

           // extra bytes at the end
           post_bytes: post_bytes_parser,

           || {
               PuzFile {
                   // extra bytes at the beginning
                   pre_bytes: pre_bytes,

                   // file header
                   file_checksum: file_checksum,
                   base_checksum: base_checksum,
                   masked_low_checksums: masked_low_checksums,
                   masked_high_checksums: masked_high_checksums,
                   version: version,
                   reserved_1c: reserved_1c,
                   scrambled_checksum: scrambled_checksum,
                   reserved_20: reserved_20,
                   width: width,
                   height: height,
                   num_clues: num_clues,
                   unknown_bitmask: unknown_bitmask,
                   scrambled_tag: scrambled_tag,

                   // file body
                   solution: solution,
                   grid: grid,
                   title: title,
                   author: author,
                   copyright: copyright,
                   clues: clues,
                   notes: notes,

                   // extra bytes at the end
                   post_bytes: post_bytes,
               }
            }
));

fn print_grid(grid: Vec<char>, width: usize, height: usize) {
    for row in 0..height {
        for col in 0..width {
            print!("{}", grid[row*height + col]);
        }
        println!("");
    }
}

fn main() {
    let file = File::open("./assets/nyt_partlyfilled.puz").unwrap();
    let bytes: Vec<u8> = file.bytes().map(|x| x.unwrap()).collect();

    let matched = full(&bytes).unwrap().1;
    print_grid(matched.solution, matched.width as usize, matched.height as usize);
    println!("");
    print_grid(matched.grid, matched.width as usize, matched.height as usize);
    println!("");
    for clue in matched.clues {
        println!("{}", clue);
    }
}
