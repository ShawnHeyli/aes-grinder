use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::fs::File;
use std::io::Read;
use std::num::ParseIntError;
use std::vec;
use log::info;
use log::debug;
use log::trace;

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

enum EndOfLineParse {
    File,
    Line,
    Comment,
}

enum EndOfTermParse {
    File,
    Line,
    Term,
    Comment,
}

pub struct Parser {
    reader: Reader,
    filename: String,
    pub vars_map: HashMap<String, usize>,
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

    /* skip_withespace
        Arguments   ::

        Description :: Skip all withespaces,
                       stop before encountering an other character

        Return      :: - true if end of file
                       - false either
     */
    fn skip_whitespaces(&mut self) -> bool {
        debug!("Parser::skip_whitespaces");
    
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

    /* pass_commentary
        Arguments   ::

        Description ::  Pass commentary,
                        FORM OF COMMENTARY :
                        __input__ #<MY_COMMENTARY># __input__
                        __input__ #<MY_COMMENTARY>END_OF_LINE
                        START_OF_LINE#<MY_COMMENTARY>END_OF_LINE

        Return      ::  - true if end of file
                        - false either
     */
    fn pass_commentary(&mut self) -> EndOfLineParse {
        debug!("Parser::pass_commentary");

        let mut oc: Option<char> = self.reader.next_char();

        loop {
            match oc {
                Some('\n') => { // END OF LINE
                    return EndOfLineParse::Line;
                }
                Some('#') => { // END OF COMMENT
                    return EndOfLineParse::Comment;
                }
                Some(_) => {
                    // Pass char of commentary
                }
                None => { // END OF FILE
                    return EndOfLineParse::File;
                }
            }

            oc = self.reader.next_char();
        }
    }

    /* get_section
        Arguments   ::

        Description ::  get the name of section and set Parser struct with.
                        section is declare like that : 
                            -- my_section:

        Return      :: Ok with
                        - true if section found
                        - false either
                       OR
                       Err with parsing error
     */
    fn get_section(&mut self) -> Result<bool, ParserError> {
        debug!("Parser::get_section");

        let mut oc: Option<char> = self.reader.next_char();

        match oc {
            Some('-') => { // first dash CHECK 
                oc = self.reader.next_char();

                match oc {
                    Some('-') => { // second dash CHECK
                        let mut section_name: String = String::new();
                        let mut reach_colon: bool = false;
                        oc = self.reader.next_char();

                        loop {
                            match oc {
                                Some('\n') | None => { // end of line | or end of file
                                    if !reach_colon {
                                        return Err(ParserError::new(
                                            self.reader.line,
                                            self.reader.char_,
                                            String::from("section name need to finish by colon"),
                                        ));
                                    }
                                    return Ok(true);
                                }
                                Some(':') => { // reach colon
                                    reach_colon = true;
                                    self.section_name = Some(section_name.clone());
                                }
                                Some(c) => { // other characteres
                                    if !reach_colon {
                                        section_name.push(c);
                                    }
                                }
                            }

                            oc = self.reader.next_char();
                        }
                    }
                    Some(_) | None => { // NO second dash
                        Err(ParserError::new(
                            self.reader.line,
                            self.reader.char_,
                            String::from("commentary need to have two dashes"),
                        ))
                    }
                }
            }
            Some(c) => { // NO first dash
                self.reader.block_next(c);
                Ok(false)
            }
            None => { // End of file
                Ok(false)
            }
        }
    }

    /* prestore_term
        Arguments   ::  - str : &str to 

        Description ::  Store in self structure, redundancy or name of term, depending
                        of the contain of str

        Return      :: Ok with
                        - true if store something into self structure
                        - false either
                       OR
                       Err with parsing error
     */
    fn prestore_term (&mut self, str: &str, is_number: bool) -> Result<(), ParserError> {
        debug!("Parser::prestore_term");

        if !str.is_empty() {
            if is_number { // BUILD REDUNDANCY OF TERM
                if !self.redundancy.is_none() {
                    return Err(ParserError::new(
                        self.reader.line,
                        self.reader.char_,
                        String::from(
                            "double redundancy in sigle term, FORBIDEN!",
                        ),
                    ));
                }
                self.redundancy = Some(
                    self.conv_str_to_integer(str.to_string())
                        .expect("Error while parsing int"),
                );
            } else {      // BUILD NAME OF TERM
                if !self.var_name.is_none() {
                    return Err(ParserError::new(
                        self.reader.line,
                        self.reader.char_,
                        String::from(
                            "double var_name in sigle term, FORBIDEN!",
                        ),
                    ));
                }
                if String::from("KV").eq(&str) { // str == KV
                    return Err(ParserError::new(
                        self.reader.line,
                        self.reader.char_,
                        String::from(
                            "'KV' term is an internal keyword and is forbidden for use in variable declarations!",
                        ),
                    ));
                }
                self.var_name = Some(str.to_string());
            }

        }

        Ok(())
    }

    /* get_term
        Arguments   ::

        Description :: browse term and set
                            self.redundancy and/or self.var_name

                       term :: + <(Redundancy) * (Var)> +
                       possible to have redundancy or var or both

        Return      :: Ok with possible EndOfTermParse
                       Err if an IO error
     */
    fn get_term(&mut self) -> Result<EndOfTermParse, ParserError> {
        debug!("Parser::get_term");

        let mut is_number: bool = true;
        let mut blank_appear_inside_str: bool = false;
        let mut str: String = String::new();

        while let Some (char_) = self.reader.next_char() {
            match char_ {
                '#' => { // START OF COMMENTARY
                    match self.pass_commentary() {
                        EndOfLineParse::File => {
                            self.prestore_term(&str, is_number)
                                .expect("Erro while prestore_term");
                            return Ok(EndOfTermParse::File);
                        }
                        EndOfLineParse::Line => {
                            self.prestore_term(&str, is_number)
                                .expect("Erro while prestore_term");
                            return Ok(EndOfTermParse::Line);
                        }
                        EndOfLineParse::Comment => {}
                    }
                },
                '+' => { // END OF TERM
                    self.prestore_term(&str, is_number)
                        .expect("Error while prestore_term");
                    return Ok(EndOfTermParse::Term);
                }
                '\n' => { // END OF LINE
                    self.prestore_term(&str, is_number)
                        .expect("Error while prestore_term");
                    return Ok(EndOfTermParse::Line);
                }
                '*' => { // MIDDLE OF TERM
                    self.prestore_term(&str, is_number)
                        .expect("Error while prestore_term");
                    str.clear(); // clean string for next
                    is_number = true; // reset for next
                }
                c => { // BUILD TERM PART - REDUNDANCY OR NAME
                    if c.is_whitespace() {
                        blank_appear_inside_str = true;
                    } else {
                        // CHECK THAT REDUNDANCY AND NAME are separate with *
                        if !str.is_empty() && blank_appear_inside_str {
                            return Err(ParserError::new(
                                self.reader.line,
                                self.reader.char_,
                                String::from(
                                    "impossible to have following strings without '*' or '+'",
                                ),
                            ));
                        }

                        if str.is_empty() {
                            // unset blank_appear at start of new string
                            blank_appear_inside_str = false;
                        }

                        if is_number && c.is_alphabetic() {
                            // set for valid prestore_term call
                            is_number = false;
                        }

                        str.push(c);
                    }
                }
            }
        }

        // NO MORE CHARACTER
        self.prestore_term(&str, is_number)
            .expect("Error while affecting string");

        return Ok(EndOfTermParse::File);
    }

    /* get_vec_ndx
        Arguments   ::

        Description ::  get the index in which column self.var_name
                        need to be stored

        Return      :: Some with the valid index
                       None if nobody in self.var_name and self_redudancy
     */
    fn get_vec_ndx(&mut self) -> Option<usize> {
        debug!("Parser::get_vec_index");

        // See if variable in term
        let str: &str = match &self.var_name {
            Some(s) => s,
            None => {
                // Empty term
                self.redundancy?;
                // Only known value :: KV
                "KV"
            }
        };

        // Search in map if variable exist
        match self.vars_map.get(str) {
            Some(&index) => Some(index),
            None => { // Create new index
                let index = self.vars_map.len();
                debug!("{} at index {}", str, index);

                self.vars_map.insert(str.to_string(), index);

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

    /* store_term
        Arguments   ::

        Description :: store the term in matrix

        Return      :: Ok
                       Err either
     */
    fn store_term(&mut self) -> Result<(), ParserError> {
        debug!("Parser::store_term");

        if let Some(index) = Parser::get_vec_ndx(self) {
            let rdd = self.redundancy.unwrap_or(1);
    
            if self.matrix_count[index] == MAX_NB_MATRIX {
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

            let line = self.matrix.len() - 1;
            self.matrix[line][index] = rdd; // SET REDUNDANCY ON MATRIX
            self.matrix_count[index] += 1;
        }

        // Reset term after storing
        self.redundancy = None;
        self.var_name = None;

        Ok(())
    }

    /* parse_line
        Arguments   ::

        Description ::  parse line of the input file 

        Return      ::  Ok with possible EndOfLineParse
                        Err either
     */
    fn parse_line(&mut self) -> Result<EndOfLineParse, ParserError> {
        debug!("Parser::parse_line");

        let mut cmpt_iter: u8 = 0;
        self.matrix.push(vec![0; self.vars_map.len()]);

        loop {
            if cmpt_iter == MAX_NB_TERM { // CHECK THE MAX NUMBER of term in ONE line
                return Err(ParserError::new(
                    self.reader.line,
                    self.reader.char_,
                    String::from("too many terms in line"),
                ));
            }

            let r_es: EndOfTermParse = self.get_term().expect("Error while getting term");
            self.store_term().expect("Error while storing term");
            
            match r_es {
                EndOfTermParse::File => {
                    if self.matrix[self.matrix.len()-1].is_empty() {
                        self.matrix.pop();
                    }
                    return Ok(EndOfLineParse::File);
                }
                EndOfTermParse::Line => {
                    if self.matrix[self.matrix.len()-1].is_empty() {
                        self.matrix.pop();
                    }
                    return Ok(EndOfLineParse::Line);
                }
                EndOfTermParse::Term |
                EndOfTermParse::Comment => {}
            }

            cmpt_iter += 1;
        }
    }

    /* parse_system
        Arguments   ::  - global_infos : &mut GlobalInfos

        Description ::  Open the file given in global_infos and parse it
                        to build the corresponding section
                        In other call try to build next section
                        and again to reach the end of file

        Return      :: Ok with the section matrix
                       OR
                       Err with parsing error
     */
    pub fn parse_system(
        &mut self,
        global_infos: &mut GlobalInfos,
    ) -> Result<Vec<Vec<u32>>, ParserError> {
        info!("Start parsing system");
        debug!("Parser::parse_system");

        if self.skip_whitespaces() { // PARSE ALL WHITESPACES
            return Err(ParserError::new(
                self.reader.line,
                self.reader.char_,
                String::from("File is empty!"),
            ));
        }

        if self.section_name.is_none() // SEARCH FOR SECTION NAME
            && !self.get_section().expect("Error while getting section") {
            return Err(ParserError::new(
                self.reader.line,
                self.reader.char_,
                String::from("No section defined! Need to start with <--my_section:>"),
            ));
        }
        let section_name = match &self.section_name {
            Some(str) => str,
            None => unreachable!(),
        };

        global_infos.sys_name = section_name.to_string();


        loop {
            if self.skip_whitespaces() {
                break;
            }

            if self.get_section() // CATCH NEW SECTION => ENDING OF SYSTEM
            .expect("Error while getting section") {
                if self.matrix.is_empty() {
                    return Err(ParserError::new(
                        self.reader.line,
                        self.reader.char_,
                        String::from("no system to parse!"),
                    ));
                }

                info!("Parsing ended with success");
                debug!("Matrix :: {:?}", self.matrix);
                return Ok(self.matrix.clone());
                

            } else { // CONTINUE TO BUILD MATRIX
                match self.parse_line().expect("Error while processing line") {
                    EndOfLineParse::File => {
                        break;
                    }
                    EndOfLineParse::Line |
                    EndOfLineParse::Comment => {}
                }
            }
        }

        // END OF FILE
        if self.matrix.is_empty() {
            return Err(ParserError::new(
                self.reader.line,
                self.reader.char_,
                String::from("no system to parse!"),
            ));
        }

        info!("Parsing ended with success");
        debug!("Matrix :: {:?}", self.matrix);
        Ok(self.matrix.clone())
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
    fn simple_00 () {
        let mut global_infos = GlobalInfos::new(String::from("test/simple_00.eqs"));
        let mut parser_mod = Parser::new(&global_infos);

        let mtr = parser_mod.parse_system(&mut global_infos).unwrap();

        assert_eq!(mtr.len(), 2);
        assert_eq!(mtr[0], [1, 1, 1]);
        assert_eq!(mtr[1], [1, 1, 1]);
    }

    #[test]
    fn simple_01 () {
        let mut global_infos = GlobalInfos::new(String::from("test/simple_01.eqs"));
        let mut parser_mod = Parser::new(&global_infos);

        let mtr = parser_mod.parse_system(&mut global_infos).unwrap();

        assert_eq!(mtr.len(), 2);
        assert_eq!(mtr[0], [1, 1, 1, 0]);
        assert_eq!(mtr[1], [0, 1, 1, 1]);
    }

    #[test]
    fn simple_02 () {
        let mut global_infos = GlobalInfos::new(String::from("test/simple_02.eqs"));
        let mut parser_mod = Parser::new(&global_infos);

        let mtr = parser_mod.parse_system(&mut global_infos).unwrap();

        assert_eq!(mtr.len(), 2);
        assert_eq!(mtr[0], [1, 1, 1, 0]);
        assert_eq!(mtr[1], [0, 1, 1, 1]);
    }

    #[test]
    fn simple_03 () {
        let mut global_infos = GlobalInfos::new(String::from("test/simple_03.eqs"));
        let mut parser_mod = Parser::new(&global_infos);

        let mtr = parser_mod.parse_system(&mut global_infos).unwrap();

        assert_eq!(mtr.len(), 2);
        assert_eq!(mtr[0], [1, 1, 1, 0]);
        assert_eq!(mtr[1], [0, 1, 1, 1]);
    }

    #[test]
    fn simple_04 () {
        let mut global_infos = GlobalInfos::new(String::from("test/simple_04.eqs"));
        let mut parser_mod = Parser::new(&global_infos);

        let mtr = parser_mod.parse_system(&mut global_infos).unwrap();

        assert_eq!(mtr.len(), 2);
        assert_eq!(mtr[0], [1, 1, 1, 0]);
        assert_eq!(mtr[1], [0, 1, 1, 1]);
    }

    #[test]
    fn simple_05 () {
        let mut global_infos = GlobalInfos::new(String::from("test/simple_05.eqs"));
        let mut parser_mod = Parser::new(&global_infos);

        let mtr = parser_mod.parse_system(&mut global_infos).unwrap();

        assert_eq!(mtr.len(), 2);
        assert_eq!(mtr[0], [1, 1, 2, 0]);
        assert_eq!(mtr[1], [0, 1, 3, 1]);
    }

    #[test]
    fn simple_06 () {
        let mut global_infos = GlobalInfos::new(String::from("test/simple_06.eqs"));
        let mut parser_mod = Parser::new(&global_infos);

        let mtr = parser_mod.parse_system(&mut global_infos).unwrap();

        assert_eq!(mtr.len(), 2);
        assert_eq!(mtr[0], [1, 1, 2, 0, 0]);
        assert_eq!(mtr[1], [0, 1, 3, 1, 20]);
    }

    #[test]
    fn simple_07 () {
        let mut global_infos = GlobalInfos::new(String::from("test/simple_07.eqs"));
        let mut parser_mod = Parser::new(&global_infos);

        let mtr = parser_mod.parse_system(&mut global_infos).unwrap();

        assert_eq!(mtr.len(), 2);
        assert_eq!(mtr[0], [1, 1, 1]);
        assert_eq!(mtr[1], [1, 1, 1]);
    }

    #[test]
    fn complex_00 () {
        let mut global_infos = GlobalInfos::new(String::from("test/complex_00.eqs"));
        let mut parser_mod = Parser::new(&global_infos);

        let mtr = parser_mod.parse_system(&mut global_infos).unwrap();

        /*
            var_01 * 100 + var_02 + var_03 + var_04 + var_05
            var_06 * 200 + var_07 + var_08 + var_09 + var_10
            var_11 * 300 + var_12 + var_13 + var_14 + var_15
         */

        assert_eq!(mtr.len(), 3);
        assert_eq!(mtr[0], [100, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(mtr[1], [0, 0, 0, 0, 0, 200, 1, 1, 1, 1, 0, 0, 0, 0, 0]);
        assert_eq!(mtr[2], [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 300, 1, 1, 1, 1]);
    }

    #[test]
    fn complex_01 () {
        let mut global_infos = GlobalInfos::new(String::from("test/complex_01.eqs"));
        let mut parser_mod = Parser::new(&global_infos);

        let mtr = parser_mod.parse_system(&mut global_infos).unwrap();

        /*
            var_01 * 100 + var_02 + var_03 + var_04 + var_05 * 0
            var_06 * 200 + var_07 + var_08 + var_09 + var_10 * 0
            var_11 * 300 + var_12 + var_13 + var_14 + var_15 * 0
         */

        assert_eq!(mtr.len(), 3);
        assert_eq!(mtr[0], [100, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(mtr[1], [0, 0, 0, 0, 0, 200, 1, 1, 1, 0, 0, 0, 0, 0, 0]);
        assert_eq!(mtr[2], [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 300, 1, 1, 1, 0]);
    }

    #[test]
    fn complex_02 () {
        let mut global_infos = GlobalInfos::new(String::from("test/complex_02.eqs"));
        let mut parser_mod = Parser::new(&global_infos);

        let mtr = parser_mod.parse_system(&mut global_infos).unwrap();

        /*
            var_start * 0 + var_01 * 0 + var_02 * 0 + var_03 * 0 + var_final * 0
            var_start * 1 + var_04 * 0 + var_05 * 0 + var_06 * 0 + var_final * 1
            var_start * 2 + var_07 * 0 + var_08 * 0 + var_09 * 0 + var_final * 2
            var_start * 3 + var_10 * 0 + var_11 * 0 + var_12 * 0 + var_final * 3
         */

        assert_eq!(mtr.len(), 4);
        assert_eq!(mtr[0], [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(mtr[1], [1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(mtr[2], [2, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(mtr[3], [3, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn complex_03 () {
        let mut global_infos = GlobalInfos::new(String::from("test/complex_03.eqs"));
        let mut parser_mod = Parser::new(&global_infos);

        let mtr = parser_mod.parse_system(&mut global_infos).unwrap();

        /* 30 characters by line : ndx 0 -> 29 */
        assert_eq!(mtr.len(), 64);
        assert_eq!(mtr[0], [ /* line 3 */
            1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[1], [ /* line 4 */
            0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[2], [ /* line 5 */
            0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[3], [ /* line 6 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[4], [ /* line 7 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[5], [ /* line 8 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[6], [ /* line 9 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[7], [ /* line 10 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[8], [ /* line 11 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[9], [ /* line 12 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[10], [ /* line 13 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[11], [ /* line 14 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[12], [ /* line 15 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[13], [ /* line 16 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[14], [ /* line 17 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[15], [ /* line 18 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[16], [ /* line 19 */
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 3, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[17], [ /* line 20 */
            0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 3, 1, 1, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[18], [ /* line 21 */
            0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 3, 1, 1,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[19], [ /* line 22 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            2, 3, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[20], [ /* line 23 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 1, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[21], [ /* line 24 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 1, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[22], [ /* line 25 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 1,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[23], [ /* line 26 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            1, 2, 3, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[24], [ /* line 27 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 2, 3, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[25], [ /* line 28 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 2, 3, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[26], [ /* line 29 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 2, 3,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[27], [ /* line 30 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            1, 1, 2, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[28], [ /* line 31 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 1, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[29], [ /* line 32 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 1, 1, 2, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[30], [ /* line 33 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 1, 1, 2,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[31], [ /* line 34 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            3, 1, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[32], [ /* line 35 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[33], [ /* line 36 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[34], [ /* line 37 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[35], [ /* line 38 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[36], [ /* line 39 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0,0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[37], [ /* line 40 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[38], [ /* line 41 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[39], [ /* line 42 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[40], [ /* line 43 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,  1, 1,
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[41], [ /* line 44 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[42], [ /* line 45 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[43], [ /* line 46 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[44], [ /* line 47 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[45], [ /* line 48 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[46], [ /* line 49 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[47], [ /* line 50 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[48], [ /* line 51 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[49], [ /* line 52 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[50], [ /* line 53 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[51], [ /* line 54 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[52], [ /* line 55 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[53], [ /* line 56 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0,
            0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[54], [ /* line 57 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[55], [ /* line 58 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0
        ]);
        assert_eq!(mtr[56], [ /* line 59 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[57], [ /* line 60 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[58], [ /* line 61 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[59], [ /* line 62 */
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0
        ]);
        assert_eq!(mtr[60], [ /* line 63 */
            0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[61], [ /* line 64 */
            0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[62], [ /* line 65 */
            0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ]);
        assert_eq!(mtr[63], [ /* line 66 */
            0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1
        ]);
    }

    #[test]
    fn process_linearity() {
        let mut global_infos = GlobalInfos::new(String::from("test/valid.eqs"));
        let mut parser_mod = Parser::new(&global_infos);

        parser_mod
            .parse_system(&mut global_infos)
            .expect("No error while parsing system");

        print!(
            "{:?}",
            get_non_linear_variables(parser_mod.vars_map.clone())
        );
    }

    #[test]
    fn process_linearity2() {
        let mut global_infos = GlobalInfos::new(String::from("test/valid.eqs"));
        let mut parser_mod = Parser::new(&global_infos);

        parser_mod
            .parse_system(&mut global_infos)
            .expect("No error while parsing system");

        eliminate_linear_variables(parser_mod);
    }
}
//todo!() //BOUGER LES TEST EN BAS

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
    //todo!() //MAKE TEST
}

fn sort_non_linear_variables(
    non_linear_variables: HashMap<String, usize>,
    matrix: Vec<Vec<u32>>,
    vars_map: HashMap<String, usize>,
) -> (Vec<Vec<u32>>, HashMap<String, usize>) {
    let mut new_matrix = matrix.clone();
    let mut new_vars_map: HashMap<String, usize> = HashMap::new();
    let mut non_linear_indexes: Vec<usize> = Vec::new();
    //Swap columns to have S(xi) following xi
    let mut next_column_index = 0;
    for (var_name, index) in non_linear_variables.iter() {
        non_linear_indexes.push(*index);
        let var_name2: String;
        if var_name.contains("S(") {
            var_name2 = format!("{}{}", &var_name[2..], &var_name[..var_name.len() - 1]);
        } else {
            //Get S(var_name)
            var_name2 = format!("S({})", var_name);
        }
        match vars_map.get(&var_name2) {
            Some(index2) => {
                //put col index2 at nextIndex and col index at nextIndex +1
                for i in 0..matrix.len() {
                    new_matrix[i][next_column_index] = matrix[i][*index2];
                    new_matrix[i][next_column_index + 1] = matrix[i][*index];
                }
                //Swap in vars_map
                new_vars_map.insert(var_name2.clone(), next_column_index);
                new_vars_map.insert(var_name.clone(), next_column_index + 1);
                next_column_index += 2;
            }
            None => {
                //put column index at nextIndex
                for i in 0..matrix.len() {
                    new_matrix[i][next_column_index] = matrix[i][*index];
                }
                new_vars_map.insert(var_name.clone(), next_column_index);
                next_column_index += 1;
            }
        }
    }
    //Add linear variables to the matrix
    for (var_name, index) in vars_map.iter() {
        if !non_linear_indexes.contains(index) {
            //put column index at nextIndex
            for i in 0..matrix.len() {
                new_matrix[i][next_column_index] = matrix[i][*index];
                new_vars_map.insert(var_name.clone(), next_column_index);
            }
            next_column_index += 1;
        }
    }
    (new_matrix, new_vars_map)
    //todo!() //MAKE TEST
}

fn eliminate_linear_variables(p: Parser) -> (Vec<Vec<u32>>, HashMap<String, usize>) {
    //Separate equations containing linear and non linear
    //non linear variables are like x and S(x), the others are linear
    let non_linear_variables = get_non_linear_variables(p.vars_map.clone());
    sort_non_linear_variables(non_linear_variables, p.matrix, p.vars_map)
    //todo!() //MAKE TEST
}

//todo!() //MAKE TEST
//todo!() //MAKE TEST
//todo!() //MAKE TEST
//todo!() //MAKE TEST