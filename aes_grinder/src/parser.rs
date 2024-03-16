use log::{debug, error, info, trace, warn};
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::fs::File;
use std::hash::Hash;
use std::io::Read;
use std::num::ParseIntError;
use std::vec;

use crate::GlobalInfos;

const MAX_NB_TERM: u8 = 5;
const MAX_NB_MATRIX: u32 = 5;

struct Reader {
    line: usize,
    char_: usize,
    flow: String,
    index: usize,
    prev_char: Option<char>,
}

impl Reader {
    fn new(filename: &String) -> Self {
        let mut flow = File::open(filename).unwrap();
        let mut us_flow = String::new();
        flow.read_to_string(&mut us_flow).unwrap();

        Reader {
            line: 1,
            char_: 1,
            flow: us_flow,
            index: 0,
            prev_char: None,
        }
    }

    /// .
    fn block_next(&mut self, prev_char: char) {
        debug!("Parser::Reader::block_next");
        self.prev_char = Some(prev_char);
    }

    fn next_char(&mut self) -> Option<char> {
        debug!("Parser::Reader::next_char");
        let oc: Option<char>;

        match self.prev_char {
            Some(c) => {
                trace!(target: "parser", "catched {} - line {} - col {}", c, self.line, self.char_);
                oc = Some(c);
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
                    Some('\n') => {
                        self.line += 1;
                        self.char_ = 1;
                    }
                    Some(c) => {
                        trace!(target: "parser", "catched {} - line {} - col {}", c, self.line, self.char_);
                        self.char_ += 1;
                    }
                    None => {}
                }
            }
        }

        oc
    }
}

#[derive(Debug)]
pub struct ParserError {
    line: usize,
    char_: usize,
    msg: String,
}

impl ParserError {
    fn new(line: usize, char_: usize, msg: String) -> Self {
        ParserError { line, char_, msg }
    }
}

impl Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "ParserError at line {} and char {} :: {}",
            self.line, self.char_, self.msg
        )
    }
}

enum EndOfParse {
    File,
    Line,
    Comment,
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
}

impl Parser {
    pub fn new(global_infos: &GlobalInfos) -> Self {
        let reader = Reader::new(&global_infos.filename_eq_sys);
        Parser {
            reader, //: Reader::new (global_infos.filename_eq_sys.clone()),
            filename: global_infos.filename_eq_sys.clone(),
            vars_map: HashMap::new(),
            section_name: None,
            matrix: vec![],
            matrix_count: vec![],
            var_name: None,
            redundancy: None,
        }
    }

    fn get_section(&mut self) -> Result<bool, ParserError> {
        debug!("Parser::get_section");

        let mut oc: Option<char> = self.reader.next_char();

        match oc {
            Some('-') => {
                oc = self.reader.next_char();

                match oc {
                    Some('-') => {
                        let mut section_name: String = String::new();
                        let mut is_reach_colon: bool = false;
                        oc = self.reader.next_char();

                        loop {
                            match oc {
                                Some('\n') | None => {
                                    if !is_reach_colon {
                                        return Err(ParserError::new(
                                            self.reader.line,
                                            self.reader.char_,
                                            String::from("section name need to finish by colon"),
                                        ));
                                    }
                                    return Ok(true);
                                }
                                Some(':') => {
                                    is_reach_colon = true;
                                    self.section_name = Some(section_name.clone());
                                }
                                Some(c) => {
                                    if !is_reach_colon {
                                        section_name.push(c);
                                    }
                                }
                            }

                            oc = self.reader.next_char();
                        }
                    }
                    // No double dashes
                    Some(_) | None => {
                        let c = self.reader.next_char().unwrap();
                        println!("final take {}", c);
                        Err(ParserError::new(
                            self.reader.line,
                            self.reader.char_,
                            String::from("commentary need to have two dashes"),
                        ))
                    }
                }
            }
            // Others than dash
            Some(c) => {
                self.reader.block_next(c);
                Ok(false)
            }
            // EndOfFile
            None => Ok(false),
        }
    }

    // retur true if end of file
    fn skip_whitespace(&mut self) -> bool {
        debug!("Parser::skip_whitespace");

        let mut oc: Option<char> = self.reader.next_char();
        loop {
            match oc {
                Some(c) => {
                    if c.is_whitespace() {
                        oc = self.reader.next_char();
                    } else {
                        self.reader.block_next(c);
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

    fn conv_str_to_integer(&self, str: String) -> Result<u32, ParserError> {
        debug!("Parser::conv_str_to_integer");
        let r_conv: Result<u32, ParseIntError> = str.parse::<u32>();

        match r_conv {
            Ok(number) => Ok(number),
            Err(e) => Err(ParserError::new(
                self.reader.line,
                self.reader.char_,
                format!("Error while parsing integer :: {}", e),
            )),
        }
    }

    /// return true if end of file
    fn pass_commentary(&mut self) -> EndOfParse {
        debug!("Parser::pass_commentary");

        let mut oc: Option<char> = self.reader.next_char();

        loop {
            match oc {
                Some('\n') => {
                    return EndOfParse::Line;
                }
                Some('#') => {
                    return EndOfParse::Comment;
                }
                Some(_) => {
                    // Pass others char
                }
                None => {
                    return EndOfParse::File;
                }
            }

            oc = self.reader.next_char();
        }
    }

    fn affect_string(&mut self, str_: &str, is_number: bool) -> Result<(), ParserError> {
        debug!("Parser::affect_string");

        if !str_.is_empty() {
            if is_number {
                self.redundancy = Some(
                    self.conv_str_to_integer(str_.to_string())
                        .expect("Error while parsing int"),
                );
            } else {
                if String::from("KV").eq(&str_) {
                    return Err(ParserError::new(
                        self.reader.line,
                        self.reader.char_,
                        String::from(
                            "'KV' term is an internal keyword and is forbidden for use in variable declarations!",
                        ),
                    ));
                }
                self.var_name = Some(str_.to_string());
            }
        }

        Ok(())
    }

    fn get_term(&mut self) -> Result<EndOfParse, ParserError> {
        debug!("Parser::get_term");

        let mut oc: Option<char> = self.reader.next_char();

        let mut is_number: bool = true;
        let mut is_blank_appear: bool = false;
        let mut str_: String = String::new();

        loop {
            match oc {
                Some('#') => match self.pass_commentary() {
                    EndOfParse::File => {
                        return Ok(EndOfParse::Line);
                    }
                    EndOfParse::Line => {
                        return Ok(EndOfParse::Line);
                    }
                    EndOfParse::Comment => {}
                },
                Some('+') => {
                    self.affect_string(&str_, is_number)
                        .expect("Error while affecting string");
                    return Ok(EndOfParse::Line);
                }
                Some('\n') => {
                    self.affect_string(&str_, is_number)
                        .expect("Error while affecting string");
                    return Ok(EndOfParse::Line);
                }
                None => {
                    self.affect_string(&str_, is_number)
                        .expect("Error while affecting string");
                    return Ok(EndOfParse::Line);
                }
                Some('*') => {
                    self.affect_string(&str_, is_number)
                        .expect("Error while affecting string");
                    str_.clear();
                }
                Some(c) => {
                    if c.is_whitespace() {
                        is_blank_appear = true;
                    } else {
                        if is_blank_appear && !str_.is_empty() {
                            return Err(ParserError::new(
                                self.reader.line,
                                self.reader.char_,
                                String::from(
                                    "impossible to have following string without '*' or '+'",
                                ),
                            ));
                        }

                        is_blank_appear = false;

                        if is_number && c.is_alphabetic() {
                            is_number = false;
                        }
                        str_.push(c);
                    }
                }
            }

            oc = self.reader.next_char();
        }
    }

    fn get_vec_index(&mut self) -> Option<usize> {
        debug!("Parser::get_vec_index");

        // See if variable in term
        let str_: &str = match &self.var_name {
            Some(s) => s,
            None => {
                // Empty term
                self.redundancy?;
                // Only known value :: KV
                "KV"
            }
        };

        // Search in map if variable exist
        match self.vars_map.get(str_) {
            Some(&index) => Some(index),
            None => {
                let index = self.vars_map.len();
                debug!("{} at index {}", str_, index);

                self.vars_map.insert(str_.to_string(), index);

                let line = self.matrix.len() - 1;
                // extend matrix
                for i in 0..=line {
                    self.matrix[i].push(0);
                }

                self.matrix_count.push(1);

                Some(index)
            }
        }
    }

    fn add_redundancy(&mut self, index: usize) {
        debug!("Parser::add_redundancy");

        let line = self.matrix.len() - 1;

        match self.redundancy {
            Some(rdd) => {
                self.matrix[line][index] = rdd;
            }
            None => {
                self.matrix[line][index] = 1;
            }
        }
    }

    fn add_term(&mut self) -> Result<(), ParserError> {
        debug!("Parser::add_term");

        if let Some(index) = Parser::get_vec_index(self) {
            if self.matrix_count[index] == MAX_NB_MATRIX {
                let rdd = self.redundancy.unwrap_or(1);
                let name = match &self.var_name {
                    Some(n) => n.clone(),
                    None => String::from("KV"),
                };

                return Err(ParserError::new(
                    self.reader.line,
                    self.reader.char_,
                    format!("term {}*{} reach this maximum occurence", rdd, name),
                ));
            }

            self.add_redundancy(index);
            self.matrix_count[index] += 1;
        }

        Ok(())
    }

    fn process_line(&mut self) -> Result<EndOfParse, ParserError> {
        debug!("Parser::process_line");
        let mut push_matrix: bool = false;
        let mut cmpt_iter: u8 = 0;

        loop {
            if cmpt_iter == MAX_NB_TERM {
                return Err(ParserError::new(
                    self.reader.line,
                    self.reader.char_,
                    String::from("too many terms in line"),
                ));
            }

            let r_es: EndOfParse = self.get_term().expect("Error while getting term");
            if (self.redundancy.is_none() || self.var_name.is_none()) && !push_matrix {
                self.matrix.push(vec![0; self.vars_map.len()]);
                debug!(
                    "Size of matrix :: {} -- size of inner matrix :: {}",
                    self.matrix.len(),
                    self.vars_map.len()
                );
                push_matrix = true;
            }

            self.add_term().expect("Error while adding term");

            // Reset term after adding
            self.redundancy = None;
            self.var_name = None;

            match r_es {
                EndOfParse::File => {
                    return Ok(EndOfParse::File);
                }
                EndOfParse::Line => {
                    return Ok(EndOfParse::Line);
                }
                EndOfParse::Comment => {}
            }

            cmpt_iter += 1;
        }
    }

    // true if there is a system to parse
    pub fn parse_system(
        &mut self,
        global_infos: &mut GlobalInfos,
    ) -> Result<Vec<Vec<u32>>, ParserError> {
        info!("Start parsing system");
        debug!("Parser::parse_system");

        if self.skip_whitespace() {
            return Err(ParserError::new(
                self.reader.line,
                self.reader.char_,
                String::from("File is empty!"),
            ));
        }

        if self.section_name.is_none() && !self.get_section().expect("Error while getting section")
        {
            return Err(ParserError::new(
                self.reader.line,
                self.reader.char_,
                String::from("no section defined! need to defined '--my_section:'"),
            ));
        }

        let section_name = match &self.section_name {
            Some(str_) => str_,
            None => unreachable!(),
        };

        global_infos.sys_name = section_name.to_string();

        loop {
            if self.skip_whitespace() {
                break;
            }

            if !self.get_section().expect("Error while getting section") {
                // Stop condition
                match self.process_line().expect("Error while processing line") {
                    EndOfParse::File => {
                        break;
                    }
                    EndOfParse::Line => {}
                    EndOfParse::Comment => {}
                }
            } else {
                if !self.matrix.is_empty() {
                    info!("Parsing ended with success");
                    debug!("Matrix :: {:?}", self.matrix);
                    return Ok(self.matrix.clone());
                }
                return Err(ParserError::new(
                    self.reader.line,
                    self.reader.char_,
                    String::from("no system to parse!"),
                ));
            }
        }

        if !self.matrix.is_empty() {
            info!("Parsing ended with success");
            debug!("Matrix :: {:?}", self.matrix);
            return Ok(self.matrix.clone());
        }

        Err(ParserError::new(
            self.reader.line,
            self.reader.char_,
            String::from("no system to parse!"),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_system() {
        let mut global_infos = GlobalInfos::new(String::from("test/empty.eqs"));
        let mut parser_mod = Parser::new(&global_infos);

        assert!(parser_mod.parse_system(&mut global_infos).is_err());
    }

    #[test]
    fn only_commentary() {
        let mut global_infos = GlobalInfos::new(String::from("test/only_comments.eqs"));
        let mut parser_mod = Parser::new(&global_infos);

        assert!(parser_mod.parse_system(&mut global_infos).is_err());
    }

    #[test]
    fn valid_system() {
        let mut global_infos = GlobalInfos::new(String::from("test/valid.eqs"));
        let mut parser_mod = Parser::new(&global_infos);

        assert!(parser_mod.parse_system(&mut global_infos).is_ok());
    }

    #[test]
    fn process_linearity() {
        let mut global_infos = GlobalInfos::new(String::from("test/valid.eqs"));
        let mut parser_mod = Parser::new(&global_infos);

        parser_mod.parse_system(&mut global_infos).expect("No error while parsing system");

        print!("{:?}", get_non_linear_variables(parser_mod.vars_map.clone()));
    }

    #[test]
    fn process_linearity2() {
        let mut global_infos = GlobalInfos::new(String::from("test/valid.eqs"));
        let mut parser_mod = Parser::new(&global_infos);

        parser_mod.parse_system(&mut global_infos).expect("No error while parsing system");

        print!("{:?}", eliminate_linear_variables(parser_mod));
    }
}

/**
 * We obtain a hash map containing variables xi and S(xi) 
 */
fn get_non_linear_variables(vars_map: HashMap<String, usize>) -> HashMap<String, usize> {
    //For each var_name in p.vars_map
    //if name contains S(x) then add to non_linear_variables x and S(x)
    let mut non_linear_variables: HashMap<String, usize> = HashMap::new();
    let mut linear_variables: HashMap<String, usize> = HashMap::new();
    for (var_name, index) in vars_map.iter() {
        if var_name.contains("S(") {
            non_linear_variables.insert(var_name.clone(), *index);
        } else {
            linear_variables.insert(var_name.clone(), *index);
        }
    }
    let non_linear_variables2 = non_linear_variables.clone();
    for (var_name, index) in non_linear_variables2.iter() {
        let mut var_name = var_name.clone();
        var_name = format!("{}{}", &var_name[2..], &var_name[..var_name.len() - 1]);
        for (var_name2, index2) in linear_variables.iter() {
            if *var_name2 == *var_name {
                non_linear_variables.insert(var_name2.clone(), *index2);
            }
        }
    }
    non_linear_variables
}

fn sort_non_linear_variables(non_linear_variables: HashMap<String, usize>, matrix: Vec<Vec<u32>>, vars_map: HashMap<String, usize>) -> (Vec<Vec<u32>>, HashMap<String, usize>) {
    let mut new_matrix = matrix.clone(); 
    let mut new_vars_map: HashMap<String, usize> = HashMap::new();
    //Swap columns to have S(xi) following xi
    let mut next_index = 0;
    let mut non_linear_indexes: Vec<usize> = Vec::new();
    while let Some((var_name, index)) = non_linear_variables.iter().next() {
        if next_index >= matrix[0].len() {
            print!("Error");
            break;
        }
        non_linear_indexes.push(*index);
        let mut var_name2 = var_name.clone();
        if var_name2.contains("S(") {
            var_name2 = format!("{}{}", &var_name2[2..], &var_name2[..var_name2.len()-1]);
            match vars_map.get(&var_name2) {
                Some(index2) => {
                    //put index2 at nextIndex and index at nextIndex +1
                    for i in 0..matrix.len() {
                        new_matrix[i][next_index] = matrix[i][*index2];
                        new_matrix[i][next_index + 1] = matrix[i][*index];
                    }
                    //Swap in vars_map
                    new_vars_map.insert(var_name2.clone(), next_index);
                    new_vars_map.insert(var_name.clone(), next_index + 1);
                    next_index += 2;
                },
                None => {
                    //put column index at nextIndex
                    for i in 0..matrix.len() {
                        new_matrix[i][next_index] = matrix[i][*index];
                    }
                    new_vars_map.insert(var_name.clone(), next_index);
                    next_index += 1;
                }
            }
            
        }else{
            //Get S(var_name)
            let var_name2 = format!("S({})", var_name);
            match vars_map.get(&var_name2) {
                Some(index2) => {
                    //put index at nextIndex and index2 at nextIndex +1
                    for i in 0..matrix.len() {
                        new_matrix[i][next_index] = matrix[i][*index];
                        new_matrix[i][next_index + 1] = matrix[i][*index2];
                    }
                    new_vars_map.insert(var_name2.clone(), next_index);
                    new_vars_map.insert(var_name.clone(), next_index + 1);
                    next_index += 2;
                },
                None => {
                    //put column from index to nextIndex
                    for i in 0..matrix.len() {
                        new_matrix[i][next_index] = matrix[i][*index];
                    }
                    new_vars_map.insert(var_name.clone(), next_index);
                    next_index += 1;
                }
            }
            
        }
    }
    //Add linear variables
    for (var_name, index) in vars_map.iter() {
        if !non_linear_indexes.contains(index) {
            for i in 0..matrix.len() {
                new_matrix[i][next_index] = matrix[i][*index];
                new_vars_map.insert(var_name.clone(), next_index);
            }
            next_index += 1;
        }
    }
    (new_matrix, new_vars_map)
}

fn eliminate_linear_variables(p: Parser) -> (Vec<Vec<u32>>, HashMap<String, usize>){
    //Separate equations containing linear and non linear 
    //non linear variables are like x and S(x), the others are linear
    let non_linear_variables = get_non_linear_variables(p.vars_map.clone());
    sort_non_linear_variables(non_linear_variables, p.matrix, p.vars_map)
}