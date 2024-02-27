use std::fs::File;
use std::io::Read;
use std::num::ParseIntError;
use std::process;
use std::collections::HashMap;

use crate::debug::Debug;
use crate::GlobalInfos;

const MAX_NB_TERM: u8 = 5;
const MAX_NB_MATRIX: u32 = 5;

struct Reader {
    line: u32,
    char_: u16,
    flow: String,
    index: usize,
    prev_char: Option<char>,
    dbg: Debug,
}

impl Reader {
    fn new (filename: &String, dbg_granul: u8) -> Self {
        let mut flow = File::open (filename).unwrap();
        let mut us_flow = String::new ();
        flow.read_to_string(&mut us_flow).unwrap();

        Reader {
            line: 1,
            char_: 1,
            flow: us_flow,
            index: 0,
            prev_char: None,
            dbg: Debug::new (dbg_granul),
        }
    }

    /// .
    fn block_next (&mut self, prev_char: char) {
        self.dbg.print(&String::from("\nParser::Reader::block_next"), 2);
        self.prev_char = Some (prev_char);
    }

    fn next_char (&mut self) -> Option<char> {
        self.dbg.print(&String::from("\nParser::Reader::next_char"), 2);

        let oc: Option<char>;

        match self.prev_char {
            Some (c) => {
                self.dbg.print (&format! ("catched {} - line {} - col {}",
                                c, self.line, self.char_), 3);

                oc = Some (c);
                self.prev_char = None;
            }
            None => {
                oc = self.flow.chars().nth(self.index);
                self.index += 1;
                /*
                oc = self.s_flow.chars().next();
                println! ("first catch {}", oc.unwrap());
                oc = self.s_flow.chars().next();
                println! ("second catch {}", oc.unwrap());
                */
                
                match oc {
                    Some ('\n') => {
                        self.line += 1;
                        self.char_ = 1;
                    }
                    Some (c) => {
                        self.dbg.print (&format! ("catched {} - line {} - col {}",
                                        c, self.line, self.char_), 3);

                        self.char_ += 1;
                    }
                    None => {}
                }
            }
        }
        
        oc
    }
}


enum EndingProcessComment {
    EndOfFile,
    EndOfLine,
    EndOfComment,
}

enum EndingProcessTerm {
    EndOfFile,
    EndOfLine,
    EndOfTerm,
}

enum EndingProcessLine {
    EndofFile,
    EndOfline,
}

pub struct Parser {
    reader: Reader,
    filename: String,
    vars_map: HashMap<String, usize>,
    section_name: Option<String>,
    matrix: Vec<Vec<u32>>,
    matrix_count: Vec<u32>,
    var_name: Option<String>,
    redundancy: Option<u32>,
    dbg: Debug,
}

impl Parser {
    pub fn new (global_infos: &GlobalInfos, dbg_granul: u8) -> Self {
        let reader = Reader::new(&global_infos.filename_eq_sys, dbg_granul);
        Parser {
            reader, //: Reader::new (global_infos.filename_eq_sys.clone()),
            filename: global_infos.filename_eq_sys.clone(),
            vars_map: HashMap::new (),
            section_name: None,
            matrix: vec! [],
            matrix_count: vec! [],
            var_name: None,
            redundancy: None,
            dbg: Debug::new (dbg_granul),
        }
    }

    fn failed_to_parse (&mut self, err: String) {
        self.dbg.print(&String::from("PARSER::failed_to_parse"), 2);

        eprintln! ("FAILED to parse -- {}:{}:{}\n{}{}",
                    self.filename, self.reader.line, self.reader.char_-1,
                   "                   ", err);
        process::exit (1);
    }

    fn get_section (&mut self) -> bool {
        self.dbg.print(&String::from("PARSER::get_section"), 2);

        let mut oc: Option<char> = self.reader.next_char();

        match oc {
            Some ('-') => {
                oc = self.reader.next_char();

                match oc {
                    Some ('-') => {
                        let mut section_name: String = String::new ();
                        let mut is_reach_colon: bool = false;
                        oc = self.reader.next_char();
                        
                        loop { 
                            match oc {
                                Some ('\n') | None => {
                                    if !is_reach_colon {
                                        self.failed_to_parse (String::from("section name need to finish by colon"));
                                    }
                                    return true;
                                }
                                Some (':') => {
                                    is_reach_colon = true;
                                    self.section_name = Some (section_name.clone ());
                                }
                                Some (c) => {
                                    if !is_reach_colon {
                                        section_name.push (c);
                                    }
                                }
                            }

                            oc = self.reader.next_char ();
                        }
                    }
                    // No double dashes
                    Some (_) | None => {
                        let c = self.reader.next_char().unwrap();
                        println! ("final take {}", c);
                        self.failed_to_parse (String::from("commentary need to have two dashes"));
                        return false;
                    }
                }
            }
            // Others than dash
            Some (c) => {
                self.reader.block_next (c);
                false
            }
            // EndOfFile
            None => {
                false
            }
        }
    }

    // retur true if end of file
    fn skip_whitespace (&mut self) -> bool {
        self.dbg.print(&String::from("PARSER::skip_whitespace"), 2);

        let mut oc: Option<char> = self.reader.next_char();
        loop {
            match oc {
                Some (c) => {
                    if c.is_whitespace() {
                        oc = self.reader.next_char();
                    } else {
                        self.reader.block_next (c);
                        break;
                    }
                }
                None => {
                    return true;
                }
            }
        }

        false
    }

    fn conv_str_to_integer (&self, str: &String) -> u32 {
        self.dbg.print(&String::from("PARSER::conv_str_to_integer"), 2);

        let r_conv: Result <u32, ParseIntError> = str.parse::<u32> ();
        
        match r_conv {
            Ok (number) => {
                number
            }
            Err (e) => {
                eprintln! ("ERROR :: fnct - get_string :: {}{} - {}",
                          "failed to parse number with string -> ", 
                          str, e);
                process::exit (1);
            }
        }
    }

    /// return true if end of file
    fn pass_commentary (&mut self) -> EndingProcessComment {
        self.dbg.print(&String::from("PARSER::pass_commentary"), 2);

        let mut oc: Option<char> = self.reader.next_char ();

        loop {
            match oc {
                Some ('\n') => {
                    return EndingProcessComment::EndOfLine;
                }
                Some ('#') => {
                    return EndingProcessComment::EndOfComment;
                }
                Some (_) => {
                    // Pass others char
                }
                None => {
                    return EndingProcessComment::EndOfFile;
                }
            }

            oc = self.reader.next_char ();
        }
    }

    fn affect_string (&mut self, str_: &String, is_number: bool) {
        self.dbg.print(&String::from("PARSER::affect_string"), 2);

        if !str_.is_empty () {
            if is_number {
                self.redundancy = Some (self.conv_str_to_integer(&str_));
            } else {
                if String::from("KV").eq(str_) {
                    self.failed_to_parse(String::from("'KV' term is an internal keyword and is forbidden for use in variable declarations!"));
                }
                self.var_name = Some (str_.clone ());
            }
        }
    }

    fn get_term (&mut self) -> EndingProcessTerm {
        self.dbg.print(&String::from("PARSER::get_term"), 2);

        let mut oc: Option<char> = self.reader.next_char();

        let mut is_number: bool = true;
        let mut is_blank_appear: bool = false;
        let mut str_: String = String::new ();
        

        loop {
            match oc {
                Some ('#') => {
                    match self.pass_commentary() {
                        EndingProcessComment::EndOfFile => {
                              return EndingProcessTerm::EndOfFile;
                        }
                        EndingProcessComment::EndOfLine => {
                            return EndingProcessTerm::EndOfLine;
                        }
                        EndingProcessComment::EndOfComment => {}
                    }
                }
                Some ('+') => {
                    self.affect_string(&str_, is_number);
                    return EndingProcessTerm::EndOfTerm;
                }
                Some ('\n') => {
                    self.affect_string(&str_, is_number);
                    return EndingProcessTerm::EndOfLine;
                }
                None => {
                    self.affect_string(&str_, is_number);
                    return EndingProcessTerm::EndOfFile;
                }
                Some ('*') => {
                    self.affect_string(&str_, is_number);
                    str_.clear ();
                }
                Some (c) => {
                    if c.is_whitespace () {
                        is_blank_appear = true;
                    }
                    
                    else {
                        if is_blank_appear && !str_.is_empty() {
                            self.failed_to_parse(String::from ("impossible to have following string without '*' or '+'"));
                        }
                        
                        is_blank_appear = false;

                        if is_number && c.is_alphabetic() {
                            is_number = false;
                        }
                        str_.push (c);
                    }
                }
            }

            oc = self.reader.next_char ();
        }
    }

    fn get_vec_index (&mut self) -> Option<usize> {
        self.dbg.print(&String::from("PARSER::get_vec_index"), 2);

        // See if variable in term
        let str_: &str = match &self.var_name {
            Some(s) => s,
            None => {
                // Empty term
                if self.redundancy == None {
                    return None;
                }
                // Only known value :: KV 
                "KV"
            }
        };

        // Search in map if variable exist
        match self.vars_map.get(str_) {
            Some(&index) => Some (index),
            None => {
                let index = self.vars_map.len();
                self.dbg.print (&format!("{} at index {}", str_, index), 1);

                self.vars_map.insert(str_.to_string(), index);
                
                let line = self.matrix.len() - 1;
                // extend matrix
                for i in 0..=line {
                    self.matrix[i].push(0);
                }

                self.matrix_count.push (1);

                Some (index)
            }
        }
    }

    fn add_redundancy (&mut self, index: usize) {
        self.dbg.print(&String::from("\nPARSER::add_redundancy"), 2);

        let line = self.matrix.len() - 1;

        match self.redundancy {
            Some (rdd) => {
                self.matrix[line][index] = rdd;
            }
            None => {
                self.matrix[line][index] = 1;
            }
        }
    }

    fn add_term (&mut self) {
        self.dbg.print(&String::from("\nPARSER::add_term"), 2);

        match self.get_vec_index () {
            Some (index) => {
                if self.matrix_count[index] == MAX_NB_MATRIX {
                    let rdd = match self.redundancy {
                        Some (r) => r,
                        None => 1,
                    };
                    let name = match &self.var_name {
                        Some (n) => n.clone(),
                        None => String::from("KV"),
                    };

                    self.failed_to_parse(format! ("term {}*{} reach this maximum occurence", 
                                                        rdd, name));
                }

                self.add_redundancy (index);
                self.matrix_count[index] += 1;
            }
            None => {}
        }
    }

    fn process_line (&mut self) -> EndingProcessLine {
        let mut push_matrix: bool = false;
        self.dbg.print(&String::from("\nPARSER::process_line"), 2);

        let mut cmpt_iter: u8 = 0;

        loop {
            if cmpt_iter == MAX_NB_TERM {
                self.failed_to_parse(format!("plus than {} terms in one line", MAX_NB_TERM));
            }

            let r_es: EndingProcessTerm = self.get_term ();
            if (self.redundancy != None || self.var_name != None) && !push_matrix {
                self.matrix.push(vec![0; self.vars_map.len()]);
                self.dbg.print (&format!("Size of matrix :: {} -- size of inner matrix :: {}", 
                        self.matrix.len(), self.vars_map.len()), 2);
                push_matrix = true;
            }
            
            self.add_term();

            // Reset term after adding
            self.redundancy = None;
            self.var_name = None;

            match r_es {
                EndingProcessTerm::EndOfFile => {
                    return EndingProcessLine::EndofFile;
                }
                EndingProcessTerm::EndOfLine => {
                    return EndingProcessLine::EndOfline;
                }
                EndingProcessTerm::EndOfTerm => {}
            }

            cmpt_iter += 1;
        }
    }

    // true if there is a system to parse
    pub fn parse_system (&mut self, global_infos: &mut GlobalInfos) -> bool {
        self.dbg.print(&String::from("\nPARSER::parse_system"), 2);

        if self.skip_whitespace() {
            println! ("No more section");
            return false;
        }

        if self.section_name == None {
            if !self.get_section() {
                self.failed_to_parse(String::from(
                    "no section defined! need to defined '--my_section:'"));
            }
        }

        let section_name = match &self.section_name {
            Some (str_) => {
                str_
            }
            None => {
                process::exit(1);
            }
        };

        global_infos.sys_name = section_name.to_string();

        loop {
            match self.skip_whitespace() {
                true => {
                    break;
                }
                false => {}
            }

            if !self.get_section() { // Stop condition
                match self.process_line() {
                    EndingProcessLine::EndofFile => {
                        break;
                    }
                    EndingProcessLine::EndOfline => {}
                }
            } else {

                if self.matrix.len() > 0 {
                    self.dbg.apply_fct(Parser::print_matrix,self, 1);
                    return true;
                }
                return false;
            }
        }

        if self.matrix.len() > 0 {
            self.dbg.apply_fct(Parser::print_matrix,self, 1);
            return true;
        }

        false
    }

    fn print_matrix (&self) {
        self.dbg.print(&String::from("\nPARSER::parse_system"), 2);
        for row in &self.matrix {
            print! ("[");
            for (i, &element) in row.iter().enumerate() {
                print!("{}{} ", element, 
                    if i < row.len() - 1 { "," } else { "" }
                );
            }
            println!("]");
        }
    }
}
