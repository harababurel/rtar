use std::io::prelude::*;
use std::fs;
use std::fs::File;
use std::io::SeekFrom;

use crate::archive::*;

pub fn extract_files(f: &mut File, action: Action) {
    loop {
        let head = UstarHeader::read_header(f);
        // TODO: checksum check
        // If filename is empty, consider archive over
        if head.file_name[0] == 0 {
            break;
        }

        head.display_file_info();

        let size = head.file_size();
        // Tar files are partitioned into 512 byte chunks,
        // padded with zeroes if data doesn't fill them 
        let chunks = (size / 512) + 1;

        if let Action::Extract = action {
            match head.file_type() {
                FileType::Normal => {
                    let mut newfile = File::create(head.file_name()).expect("ERROR creating file");
                    for i in 0..chunks {
                        let mut buffer: [u8; 512] = [0; 512];
                        f.read(&mut buffer).expect("ERROR buffer overflow");
                        // The last chunk is padded with zeroes, but they mustn't be written to file
                        if i == chunks - 1 {
                            newfile.write_all(&buffer[0..(size - (chunks - 1) * 512)]).expect("ERROR writing file");
                        }
                        else {
                            newfile.write_all(&buffer[..]).expect("ERROR writing file");
                        }
                    }
                },
                FileType::Directory => {
                    fs::create_dir(head.file_name()).expect("ERROR creating directory");
                }
                _ => panic!("Filetype not implemeted yet")

            }
        }
        else if let Action::Display = action {
            if let FileType::Normal = head.file_type() {
                // Jump the contents chunks if just displaying info
                f.seek(SeekFrom::Current((chunks * 512) as i64)).expect("ERROR seek file");
            }
        }

        
        println!();
    }
}

pub fn archive_files(f: &mut File, files: Vec<String>) {
    files.iter().for_each(|file_name| {
        let mut file = File::open(file_name).expect("ERROR opening file");
        let header = UstarHeader::header_from_file(&file, file_name);
        let size = header.file_size();
        let chunks = (size / 512) + 1;

        f.write_all(&header.serialize_to_array()).expect("ERROR writing to file");

        if let FileType::Normal = header.file_type() {
            for i in 0..chunks {
                let mut buffer: [u8; 512] = [0; 512];
                let _ = file.read(&mut buffer).expect("ERROR couldn't read file");
                f.write_all(&buffer).expect("ERROR couldn't write to file");
            }
        }
        else if let FileType::Directory = header.file_type() {
            let contents = fs::read_dir(file_name).unwrap();
            let mut paths: Vec<String> = Vec::new();

            for path in contents {
                // Ugly, but it seems to be the standard Rust way to read directory contents
                paths.push(path.unwrap().path().display().to_string().clone());
            }
            
            // Recursively call the function for the directory contents
            archive_files(f, paths);
        }
    });
}

#[derive(Debug)]
pub enum Action {
    Extract,
    Display,
    Archive,
    Nop
}