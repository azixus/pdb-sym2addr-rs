use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Seek, SeekFrom};

use pdb::FallibleIterator;

fn read_bytes_until_null(filename: &str, offset: u64) -> io::Result<Vec<u8>> {
    let mut file = BufReader::new(File::open(filename)?);
    file.seek(SeekFrom::Start(offset))?;

    let mut buffer = Vec::new();
    file.read_until(b'\x00', &mut buffer)?;
    if buffer.last() == Some(&0) {
        buffer.pop();
    }

    Ok(buffer)
}

fn print_row(offset: u32, name: pdb::RawString<'_>, val: &str) {
    println!(
        "{:x},{},{}",
        offset, name, val
    );
}

fn dump_syms(exe_filename: &str, pdb_filename: &str, sym_filters: Vec<&str>) -> pdb::Result<()> {
    let pdb = std::fs::File::open(pdb_filename)?;
    let mut pdb = pdb::PDB::open(pdb)?;
    let symbol_table = pdb.global_symbols()?;
    let sections = pdb.sections().unwrap().unwrap();

    println!("file_offset,name,value");

    let mut hits = vec![false; sym_filters.len()];
    let mut symbols = symbol_table.iter();
    while let Some(symbol) = symbols.next()? {
        match symbol.parse()? {
            pdb::SymbolData::Data(data) => {
                if let Some(idx) = sym_filters.iter().position(|&s| s == data.name.to_string()) {
                    if hits[idx] == false {
                        let offset = data.offset.offset + sections[(data.offset.section - 1) as usize].pointer_to_raw_data;
                        let val = read_bytes_until_null(exe_filename, offset as u64)?;

                        print_row(offset, data.name, std::str::from_utf8(&val).unwrap());
                        hits[idx] = true;
                    }
                }

                if hits.iter().all(|h| *h) {
                    break;
                }
            }
            _ => {}
        }    
    }

    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Usage: {}  <file.exe> <file.pdb <filter1> <filter2> ...", args[0]);
        return;
    }

    let exe = &args[1];
    let pdb = &args[2];
    let filters: Vec<&str> = args[3..].iter().map(|s| s.as_str()).collect();

    match dump_syms(exe, pdb, filters) {
        Ok(_) => (),
        Err(e) => eprintln!("error dumping PDB: {}", e),
    }
}
