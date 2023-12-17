use std::{
    collections::HashMap,
    fs::{self, DirEntry},
    io,
    process::exit, path::PathBuf,
};

use xml::reader::{EventReader, XmlEvent};

#[derive(Debug)]
struct Lexer<'a> {
    content: &'a [char],
}

impl<'a> Lexer<'a> {
    fn new(content: &'a [char]) -> Self {
        Lexer { content }
    }

    fn trim_left(&mut self) {
        while self.content.len() > 0 && self.content[0].is_whitespace() {
            self.content = &self.content[1..];
        }
    }

    fn chop(&mut self, n: usize) -> &'a [char] {
        let token = &self.content[0..n];
        self.content = &self.content[n..];
        return token;
    }

    fn chop_while<P>(&mut self, mut predicate: P) -> &'a [char]
    where
        P: FnMut(&char) -> bool,
    {
        let mut n = 0;
        while self.content.len() > 0 && predicate(&self.content[n]) {
            n += 1;
        }
        return self.chop(n);
    }

    fn next_token(&mut self) -> Option<&'a [char]> {
        self.trim_left();
        if self.content.len() == 0 {
            return None;
        }

        if self.content[0].is_numeric() {
            return Some(self.chop_while(|x| x.is_numeric()));
        }

        if self.content[0].is_alphabetic() {
            return Some(self.chop_while(|x| x.is_alphanumeric()));
        }

        return Some(self.chop(1));
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = &'a [char];
    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

fn read_xml_file_into_string(file_path: &str) -> io::Result<String> {
    println!("file_path = {file_path}");
    let file = fs::File::open(file_path).unwrap_or_else(|err| {
        eprintln!("ERROR: could not read {file_path}: {err}");
        exit(1);
    });

    let er = EventReader::new(file);

    let mut content = String::new();
    for event in er.into_iter() {
        let event = event.unwrap_or_else(|err| {
            eprintln!("ERR: could not get next event from event reader: {err}");
            exit(1);
        });

        if let XmlEvent::Characters(text) = event {
            content.push_str(&text);
            content.push_str(" ");
        }
    }

    Ok(content)
}

type TermFreq = HashMap::<String, usize>;
type TermFreqIndex = HashMap::<PathBuf, TermFreq>;

fn main() -> io::Result<()> {
    let index_path = "index.json";
    let index_file = fs::File::open(index_path)?;
    println!("Reading {index_path}...");
    let global_index: TermFreqIndex = serde_json::from_reader(index_file).expect("serde does not fail");
    println!("{index_path} contains {count} files", count = global_index.len());
    Ok(())
}

fn main2() {
    let file_path = "docs.gl/gl4/glClear.xhtml";

    let content = read_xml_file_into_string(file_path as &str)
        .unwrap()
        .chars()
        .collect::<Vec<_>>();

    let mut tf = TermFreq::new();
    let mut globalIndex = TermFreqIndex::new();

    for token in Lexer::new(&content) {
        let term = token
            .iter()
            .map(|x| x.to_ascii_uppercase())
            .collect::<String>();

        if let Some(freq) = tf.get_mut(&term) {
            *freq += 1;
        } else {
            tf.insert(term, 1);
        }
        // println!("{term}");
    }

    let mut stats = tf.iter().collect::<Vec<_>>();
    stats.sort_by_key(|(_, f)| *f);
    stats.reverse();

    for (t, f) in stats.iter().take(10) {
        println!("{} => {}", t, f);
    }

    let dir_path = "docs.gl/gl4/";
    let dir = fs::read_dir(dir_path);

    if let Ok(dir) = dir {
        println!("{:?}", dir);
        for file in dir {
            let file_path = file.as_ref().unwrap().file_name();
            let file_pathbuf = file.unwrap().path();
            let file_name = file_path.to_str().unwrap();
            let final_path = format!("docs.gl/gl4/{file_name}");
            println!("indexing {final_path}...");
            let file_content =
                read_xml_file_into_string(&final_path[..]).unwrap_or_else(|err| "Err".to_string()).chars().collect::<Vec<_>>();

            //
            let mut tf = HashMap::<String, usize>::new();

            for token in Lexer::new(&file_content) {
                let term = token
                    .iter()
                    .map(|x| x.to_ascii_uppercase())
                    .collect::<String>();

                if let Some(freq) = tf.get_mut(&term) {
                    *freq += 1;
                } else {
                    tf.insert(term, 1);
                }
                // println!("{term}");
            }

            let mut stats = tf.iter().collect::<Vec<_>>();
            stats.sort_by_key(|(_, f)| *f);
            stats.reverse();

            // for (t, f) in stats.iter().take(10) {
            //     println!("{} => {}", t, f);
            // }
            
            globalIndex.insert(file_pathbuf, tf);
            //
            // println!("{file_path:?} -> {size}", size = file_content.len());
            // println!("----------------------------------");
        }
    }

    for (k, v) in globalIndex.iter() {
        println!("{:?} has {} unique tokens", k, v.len());
    }

    let index_filepath = "index.json";
    println!("saving {index_filepath}...");
    let index_file = fs::File::create(index_filepath).unwrap();
    serde_json::to_writer(index_file, &globalIndex).expect("serde works fine");

}
