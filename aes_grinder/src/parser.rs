use crate::matrix::Matrix;
use crate::GlobalInfos;
use log::debug;
use log::info;
use log::trace;
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::fs::File;
use std::io::Read;
use std::num::ParseIntError;
use std::vec;

const MAX_NB_TERM: u8 = 20;
const MAX_NB_MATRIX: u32 = 10;

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
}

pub struct Parser {
    reader: Reader,
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
                Some('\n') => {
                    // END OF LINE
                    return EndOfLineParse::Line;
                }
                Some('#') => {
                    // END OF COMMENT
                    return EndOfLineParse::Comment;
                }
                Some(_) => {
                    // Pass char of commentary
                }
                None => {
                    // END OF FILE
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
            Some('-') => {
                // first dash CHECK
                oc = self.reader.next_char();

                match oc {
                    Some('-') => {
                        // second dash CHECK
                        let mut section_name: String = String::new();
                        let mut reach_colon: bool = false;
                        oc = self.reader.next_char();

                        loop {
                            match oc {
                                Some('\n') | None => {
                                    // end of line | or end of file
                                    if !reach_colon {
                                        return Err(ParserError::new(
                                            self.reader.line,
                                            self.reader.char_,
                                            String::from("section name need to finish by colon"),
                                        ));
                                    }
                                    return Ok(true);
                                }
                                Some(':') => {
                                    // reach colon
                                    reach_colon = true;
                                    self.section_name = Some(section_name.clone());
                                }
                                Some(c) => {
                                    // other characteres
                                    if !reach_colon {
                                        section_name.push(c);
                                    }
                                }
                            }

                            oc = self.reader.next_char();
                        }
                    }
                    Some(_) | None => {
                        // NO second dash
                        Err(ParserError::new(
                            self.reader.line,
                            self.reader.char_,
                            String::from("commentary need to have two dashes"),
                        ))
                    }
                }
            }
            Some(c) => {
                // NO first dash
                self.reader.block_next(c);
                Ok(false)
            }
            None => {
                // End of file
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
    fn prestore_term(&mut self, str: &str, is_number: bool) -> Result<(), ParserError> {
        debug!("Parser::prestore_term");

        if !str.is_empty() {
            if is_number {
                // BUILD REDUNDANCY OF TERM
                if self.redundancy.is_some() {
                    return Err(ParserError::new(
                        self.reader.line,
                        self.reader.char_,
                        String::from("double redundancy in sigle term, FORBIDEN!"),
                    ));
                }
                self.redundancy = Some(
                    self.conv_str_to_integer(str.to_string())
                        .expect("Error while parsing int"),
                );
            } else {
                // BUILD NAME OF TERM
                if self.var_name.is_some() {
                    return Err(ParserError::new(
                        self.reader.line,
                        self.reader.char_,
                        String::from("double var_name in sigle term, FORBIDEN!"),
                    ));
                }
                if String::from("KV").eq(&str) {
                    // str == KV
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

        while let Some(char_) = self.reader.next_char() {
            match char_ {
                '#' => {
                    // START OF COMMENTARY
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
                }
                '+' => {
                    // END OF TERM
                    self.prestore_term(&str, is_number)
                        .expect("Error while prestore_term");
                    return Ok(EndOfTermParse::Term);
                }
                '\n' => {
                    // END OF LINE
                    self.prestore_term(&str, is_number)
                        .expect("Error while prestore_term");
                    return Ok(EndOfTermParse::Line);
                }
                '*' => {
                    // MIDDLE OF TERM
                    self.prestore_term(&str, is_number)
                        .expect("Error while prestore_term");
                    str.clear(); // clean string for next
                    is_number = true; // reset for next
                }
                c => {
                    // BUILD TERM PART - REDUNDANCY OR NAME
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

        Ok(EndOfTermParse::File)
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
            None => {
                // Create new index
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
    fn store_term(&mut self) -> Result<bool, ParserError> {
        debug!("Parser::store_term");

        if self.redundancy.is_none() && self.var_name.is_none() {
            return Ok(false);
        }

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

        Ok(true)
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
        let mut add_term: bool = false;
        self.matrix.push(vec![0; self.vars_map.len()]);

        loop {
            if cmpt_iter == MAX_NB_TERM {
                // CHECK THE MAX NUMBER of term in ONE line
                return Err(ParserError::new(
                    self.reader.line,
                    self.reader.char_,
                    String::from("too many terms in line"),
                ));
            }

            let r_es: EndOfTermParse = self.get_term().expect("Error while getting term");
            add_term = self.store_term().expect("Error while storing term");

            match r_es {
                EndOfTermParse::File => {
                    if !add_term {
                        self.matrix.pop();
                    }
                    return Ok(EndOfLineParse::File);
                }
                EndOfTermParse::Line => {
                    if !add_term {
                        self.matrix.pop();
                    }
                    return Ok(EndOfLineParse::Line);
                }
                EndOfTermParse::Term => {}
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
    pub fn parse_system(&mut self, global_infos: &mut GlobalInfos) -> Result<Matrix, ParserError> {
        info!("Start parsing system");
        debug!("Parser::parse_system");

        if self.skip_whitespaces() {
            // PARSE ALL WHITESPACES
            return Err(ParserError::new(
                self.reader.line,
                self.reader.char_,
                String::from("File is empty!"),
            ));
        }

        if self.section_name.is_none() // SEARCH FOR SECTION NAME
            && !self.get_section().expect("Error while getting section")
        {
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

            if self
                .get_section() // CATCH NEW SECTION => ENDING OF SYSTEM
                .expect("Error while getting section")
            {
                if self.matrix.is_empty() {
                    return Err(ParserError::new(
                        self.reader.line,
                        self.reader.char_,
                        String::from("no system to parse!"),
                    ));
                }

                info!("Parsing ended with success");
                debug!("Matrix :: {:?}", self.matrix);
                let matrix = &self.matrix;
                let vars_map = &self.vars_map;
                return Ok(Matrix::new_from_vec(
                    matrix.to_vec(),
                    vars_map.clone(),
                    global_infos.polynomial,
                ));
            } else {
                // CONTINUE TO BUILD MATRIX
                match self.parse_line().expect("Error while processing line") {
                    EndOfLineParse::File => {
                        break;
                    }
                    EndOfLineParse::Line | EndOfLineParse::Comment => {}
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
        let matrix = &self.matrix;
        let vars_map = &self.vars_map;
        Ok(Matrix::new_from_vec(
            matrix.to_vec(),
            vars_map.clone(),
            global_infos.polynomial,
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
    fn simple_00() {
        let mut global_infos = GlobalInfos::new(String::from("test/simple_00.eqs"));
        let mut parser_mod = Parser::new(&global_infos);

        let mtr = parser_mod.parse_system(&mut global_infos).unwrap();

        assert_eq!(mtr.get_row_number(), 2);
        assert_eq!(mtr.get_row(0), [1.into(), 1.into(), 1.into()]);
        assert_eq!(mtr.get_row(1), [1.into(), 1.into(), 1.into()]);
    }

    #[test]
    fn simple_01() {
        let mut global_infos = GlobalInfos::new(String::from("test/simple_01.eqs"));
        let mut parser_mod = Parser::new(&global_infos);

        let mtr = parser_mod.parse_system(&mut global_infos).unwrap();

        assert_eq!(mtr.get_row_number(), 2);
        assert_eq!(mtr.get_row(0), [1.into(), 1.into(), 1.into(), 0.into()]);
        assert_eq!(mtr.get_row(1), [0.into(), 1.into(), 1.into(), 1.into()]);
    }

    #[test]
    fn simple_02() {
        let mut global_infos = GlobalInfos::new(String::from("test/simple_02.eqs"));
        let mut parser_mod = Parser::new(&global_infos);

        let mtr = parser_mod.parse_system(&mut global_infos).unwrap();

        assert_eq!(mtr.get_row_number(), 2);
        assert_eq!(mtr.get_row(0), [1.into(), 1.into(), 1.into(), 0.into()]);
        assert_eq!(mtr.get_row(1), [0.into(), 1.into(), 1.into(), 1.into()]);
    }

    #[test]
    fn simple_03() {
        let mut global_infos = GlobalInfos::new(String::from("test/simple_03.eqs"));
        let mut parser_mod = Parser::new(&global_infos);

        let mtr = parser_mod.parse_system(&mut global_infos).unwrap();

        assert_eq!(mtr.get_row_number(), 2);
        assert_eq!(mtr.get_row(0), [1.into(), 1.into(), 1.into(), 0.into()]);
        assert_eq!(mtr.get_row(1), [0.into(), 1.into(), 1.into(), 1.into()]);
    }

    #[test]
    fn simple_04() {
        let mut global_infos = GlobalInfos::new(String::from("test/simple_04.eqs"));
        let mut parser_mod = Parser::new(&global_infos);

        let mtr = parser_mod.parse_system(&mut global_infos).unwrap();

        assert_eq!(mtr.get_row_number(), 2);
        assert_eq!(mtr.get_row(0), [1.into(), 1.into(), 1.into(), 0.into()]);
        assert_eq!(mtr.get_row(1), [0.into(), 1.into(), 1.into(), 1.into()]);
    }

    #[test]
    fn simple_05() {
        let mut global_infos = GlobalInfos::new(String::from("test/simple_05.eqs"));
        let mut parser_mod = Parser::new(&global_infos);

        let mtr = parser_mod.parse_system(&mut global_infos).unwrap();

        assert_eq!(mtr.get_row_number(), 2);
        assert_eq!(mtr.get_row(0), [1.into(), 1.into(), 2.into(), 0.into()]);
        assert_eq!(mtr.get_row(1), [0.into(), 1.into(), 3.into(), 1.into()]);
    }

    #[test]
    fn simple_06() {
        let mut global_infos = GlobalInfos::new(String::from("test/simple_06.eqs"));
        let mut parser_mod = Parser::new(&global_infos);

        let mtr = parser_mod.parse_system(&mut global_infos).unwrap();

        assert_eq!(mtr.get_row_number(), 2);
        assert_eq!(
            mtr.get_row(0),
            [1.into(), 1.into(), 2.into(), 0.into(), 0.into()]
        );
        assert_eq!(
            mtr.get_row(1),
            [0.into(), 1.into(), 3.into(), 1.into(), 20.into()]
        );
    }

    #[test]
    fn simple_07() {
        let mut global_infos = GlobalInfos::new(String::from("test/simple_07.eqs"));
        let mut parser_mod = Parser::new(&global_infos);

        let mtr = parser_mod.parse_system(&mut global_infos).unwrap();

        assert_eq!(mtr.get_row_number(), 2);
        assert_eq!(mtr.get_row(0), [1.into(), 1.into(), 1.into()]);
        assert_eq!(mtr.get_row(1), [1.into(), 1.into(), 1.into()]);
    }

    #[test]
    fn complex_00() {
        let mut global_infos = GlobalInfos::new(String::from("test/complex_00.eqs"));
        let mut parser_mod = Parser::new(&global_infos);

        let mtr = parser_mod.parse_system(&mut global_infos).unwrap();

        /*
           var_01 * 100 + var_02 + var_03 + var_04 + var_05
           var_06 * 200 + var_07 + var_08 + var_09 + var_10
           var_11 * 300 + var_12 + var_13 + var_14 + var_15
        */

        assert_eq!(mtr.get_row_number(), 3);
        assert_eq!(
            mtr.get_row(0),
            [
                100.into(),
                1.into(),
                1.into(),
                1.into(),
                1.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into()
            ]
        );
        assert_eq!(
            mtr.get_row(1),
            [
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                200.into(),
                1.into(),
                1.into(),
                1.into(),
                1.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into()
            ]
        );
        assert_eq!(
            mtr.get_row(2),
            [
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                255.into(),
                1.into(),
                1.into(),
                1.into(),
                1.into()
            ]
        );
    }

    #[test]
    fn complex_01() {
        let mut global_infos = GlobalInfos::new(String::from("test/complex_01.eqs"));
        let mut parser_mod = Parser::new(&global_infos);

        let mtr = parser_mod.parse_system(&mut global_infos).unwrap();

        /*
           var_01 * 100 + var_02 + var_03 + var_04 + var_05 * 0
           var_06 * 200 + var_07 + var_08 + var_09 + var_10 * 0
           var_11 * 300 + var_12 + var_13 + var_14 + var_15 * 0
        */

        assert_eq!(mtr.get_row_number(), 3);
        assert_eq!(
            mtr.get_row(0),
            [
                100.into(),
                1.into(),
                1.into(),
                1.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into()
            ]
        );
        assert_eq!(
            mtr.get_row(1),
            [
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                200.into(),
                1.into(),
                1.into(),
                1.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into()
            ]
        );
        assert_eq!(
            mtr.get_row(2),
            [
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                255.into(),
                1.into(),
                1.into(),
                1.into(),
                0.into()
            ]
        );
    }

    #[test]
    fn complex_02() {
        let mut global_infos = GlobalInfos::new(String::from("test/complex_02.eqs"));
        let mut parser_mod = Parser::new(&global_infos);

        let mtr = parser_mod.parse_system(&mut global_infos).unwrap();

        /*
           var_start * 0 + var_01 * 0 + var_02 * 0 + var_03 * 0 + var_final * 0
           var_start * 1 + var_04 * 0 + var_05 * 0 + var_06 * 0 + var_final * 1
           var_start * 2 + var_07 * 0 + var_08 * 0 + var_09 * 0 + var_final * 2
           var_start * 3 + var_10 * 0 + var_11 * 0 + var_12 * 0 + var_final * 3
        */

        assert_eq!(mtr.get_row_number(), 4);
        assert_eq!(
            mtr.get_row(0),
            [
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into()
            ]
        );
        assert_eq!(
            mtr.get_row(1),
            [
                1.into(),
                0.into(),
                0.into(),
                0.into(),
                1.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into()
            ]
        );
        assert_eq!(
            mtr.get_row(2),
            [
                2.into(),
                0.into(),
                0.into(),
                0.into(),
                2.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into()
            ]
        );
        assert_eq!(
            mtr.get_row(3),
            [
                3.into(),
                0.into(),
                0.into(),
                0.into(),
                3.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into(),
                0.into()
            ]
        );
    }

    #[test]
    fn complex_03() {
        let mut global_infos = GlobalInfos::new(String::from("test/complex_03.eqs"));
        let mut parser_mod = Parser::new(&global_infos);

        let mtr = parser_mod.parse_system(&mut global_infos).unwrap();

        assert_eq!(mtr.get_row_number(), 64);

        assert_eq!(
            /* line 3 */
            mtr.get_row(0),
            [vec![1.into(); 3], vec![0.into(); 114]].concat()
        );

        assert_eq!(
            /* line 4 */
            mtr.get_row(1),
            [vec![0.into(); 3], vec![1.into(); 3], vec![0.into(); 111]].concat()
        );

        assert_eq!(
            /* line 5 */
            mtr.get_row(2),
            [vec![0.into(); 6], vec![1.into(); 3], vec![0.into(); 108]].concat()
        );

        assert_eq!(
            /* line 6 */
            mtr.get_row(3),
            [vec![0.into(); 9], vec![1.into(); 3], vec![0.into(); 105]].concat()
        );
        assert_eq!(
            /* line 7 */
            mtr.get_row(4),
            [vec![0.into(); 12], vec![1.into(); 3], vec![0.into(); 102]].concat()
        );
        assert_eq!(
            /* line 8 */
            mtr.get_row(5),
            [vec![0.into(); 15], vec![1.into(); 3], vec![0.into(); 99]].concat()
        );
        assert_eq!(
            /* line 9 */
            mtr.get_row(6),
            [vec![0.into(); 18], vec![1.into(); 3], vec![0.into(); 96]].concat()
        );
        assert_eq!(
            /* line 10 */
            mtr.get_row(7),
            [vec![0.into(); 21], vec![1.into(); 3], vec![0.into(); 93]].concat()
        );
        assert_eq!(
            /* line 11 */
            mtr.get_row(8),
            [vec![0.into(); 24], vec![1.into(); 3], vec![0.into(); 90]].concat()
        );
        assert_eq!(
            /* line 12 */
            mtr.get_row(9),
            [vec![0.into(); 27], vec![1.into(); 3], vec![0.into(); 87]].concat()
        );
        assert_eq!(
            /* line 13 */
            mtr.get_row(10),
            [vec![0.into(); 30], vec![1.into(); 3], vec![0.into(); 84]].concat()
        );
        assert_eq!(
            /* line 14 */
            mtr.get_row(11),
            [vec![0.into(); 33], vec![1.into(); 3], vec![0.into(); 81]].concat()
        );
        assert_eq!(
            /* line 15 */
            mtr.get_row(12),
            [vec![0.into(); 36], vec![1.into(); 3], vec![0.into(); 78]].concat()
        );
        assert_eq!(
            /* line 16 */
            mtr.get_row(13),
            [vec![0.into(); 39], vec![1.into(); 3], vec![0.into(); 75]].concat()
        );
        assert_eq!(
            /* line 17 */
            mtr.get_row(14),
            [vec![0.into(); 42], vec![1.into(); 3], vec![0.into(); 72]].concat()
        );
        assert_eq!(
            /* line 18 */
            mtr.get_row(15),
            [vec![0.into(); 45], vec![1.into(); 3], vec![0.into(); 69]].concat()
        );
        assert_eq!(
            /* line 19 */
            mtr.get_row(16),
            [
                vec![1.into()],
                vec![0.into(); 47],
                vec![2.into(), 3.into(), 1.into(), 1.into()],
                vec![0.into(); 65]
            ]
            .concat()
        );
        assert_eq!(
            /* line 20 */
            mtr.get_row(17),
            [
                vec![0.into(); 3],
                vec![1.into()],
                vec![0.into(); 48],
                vec![2.into(), 3.into(), 1.into(), 1.into()],
                vec![0.into(); 61]
            ]
            .concat()
        );
        assert_eq!(
            /* line 21 */
            mtr.get_row(18),
            [
                vec![0.into(); 6],
                vec![1.into()],
                vec![0.into(); 49],
                vec![2.into(), 3.into(), 1.into(), 1.into()],
                vec![0.into(); 57]
            ]
            .concat()
        );
        assert_eq!(
            /* line 22 */
            mtr.get_row(19),
            [
                vec![0.into(); 9],
                vec![1.into()],
                vec![0.into(); 50],
                vec![2.into(), 3.into(), 1.into(), 1.into()],
                vec![0.into(); 53]
            ]
            .concat()
        );
        assert_eq!(
            /* line 23 */
            mtr.get_row(20),
            [
                vec![0.into(); 12],
                vec![1.into()],
                vec![0.into(); 35],
                vec![1.into(), 2.into(), 3.into(), 1.into()],
                vec![0.into(); 65]
            ]
            .concat()
        );
        assert_eq!(
            /* line 24 */
            mtr.get_row(21),
            [
                vec![0.into(); 15],
                vec![1.into()],
                vec![0.into(); 36],
                vec![1.into(), 2.into(), 3.into(), 1.into()],
                vec![0.into(); 61]
            ]
            .concat()
        );
        assert_eq!(
            /* line 25 */
            mtr.get_row(22),
            [
                vec![0.into(); 18],
                vec![1.into()],
                vec![0.into(); 37],
                vec![1.into(), 2.into(), 3.into(), 1.into()],
                vec![0.into(); 57]
            ]
            .concat()
        );
        assert_eq!(
            /* line 26 */
            mtr.get_row(23),
            [
                vec![0.into(); 21],
                vec![1.into()],
                vec![0.into(); 38],
                vec![1.into(), 2.into(), 3.into(), 1.into()],
                vec![0.into(); 53]
            ]
            .concat()
        );
        assert_eq!(
            /* line 27 */
            mtr.get_row(24),
            [
                vec![0.into(); 24],
                vec![1.into()],
                vec![0.into(); 23],
                vec![1.into(), 1.into(), 2.into(), 3.into()],
                vec![0.into(); 65]
            ]
            .concat()
        );
        assert_eq!(
            /* line 28 */
            mtr.get_row(25),
            [
                vec![0.into(); 27],
                vec![1.into()],
                vec![0.into(); 24],
                vec![1.into(), 1.into(), 2.into(), 3.into()],
                vec![0.into(); 61]
            ]
            .concat()
        );
        assert_eq!(
            /* line 29 */
            mtr.get_row(26),
            [
                vec![0.into(); 30],
                vec![1.into()],
                vec![0.into(); 25],
                vec![1.into(), 1.into(), 2.into(), 3.into()],
                vec![0.into(); 57]
            ]
            .concat()
        );
        assert_eq!(
            /* line 30 */
            mtr.get_row(27),
            [
                vec![0.into(); 33],
                vec![1.into()],
                vec![0.into(); 26],
                vec![1.into(), 1.into(), 2.into(), 3.into()],
                vec![0.into(); 53]
            ]
            .concat()
        );
        assert_eq!(
            /* line 31 */
            mtr.get_row(28),
            [
                vec![0.into(); 36],
                vec![1.into()],
                vec![0.into(); 11],
                vec![3.into(), 1.into(), 1.into(), 2.into()],
                vec![0.into(); 65]
            ]
            .concat()
        );
        assert_eq!(
            /* line 32 */
            mtr.get_row(29),
            [
                vec![0.into(); 39],
                vec![1.into()],
                vec![0.into(); 12],
                vec![3.into(), 1.into(), 1.into(), 2.into()],
                vec![0.into(); 61]
            ]
            .concat()
        );
        assert_eq!(
            /* line 33 */
            mtr.get_row(30),
            [
                vec![0.into(); 42],
                vec![1.into()],
                vec![0.into(); 13],
                vec![3.into(), 1.into(), 1.into(), 2.into()],
                vec![0.into(); 57]
            ]
            .concat()
        );
        assert_eq!(
            /* line 34 */
            mtr.get_row(31),
            [
                vec![0.into(); 45],
                vec![1.into()],
                vec![0.into(); 14],
                vec![3.into(), 1.into(), 1.into(), 2.into()],
                vec![0.into(); 53]
            ]
            .concat()
        );
        assert_eq!(
            /* line 35 */
            mtr.get_row(32),
            [vec![0.into(); 64], vec![1.into(); 3], vec![0.into(); 50]].concat()
        );
        assert_eq!(
            /* line 36 */
            mtr.get_row(33),
            [vec![0.into(); 67], vec![1.into(); 3], vec![0.into(); 47]].concat()
        );
        assert_eq!(
            /* line 37 */
            mtr.get_row(34),
            [vec![0.into(); 70], vec![1.into(); 3], vec![0.into(); 44]].concat()
        );
        assert_eq!(
            /* line 38 */
            mtr.get_row(35),
            [vec![0.into(); 73], vec![1.into(); 3], vec![0.into(); 41]].concat()
        );
        assert_eq!(
            /* line 39 */
            mtr.get_row(36),
            [vec![0.into(); 76], vec![1.into(); 3], vec![0.into(); 38]].concat()
        );
        assert_eq!(
            /* line 40 */
            mtr.get_row(37),
            [vec![0.into(); 79], vec![1.into(); 3], vec![0.into(); 35]].concat()
        );
        assert_eq!(
            /* line 41 */
            mtr.get_row(38),
            [vec![0.into(); 82], vec![1.into(); 3], vec![0.into(); 32]].concat()
        );
        assert_eq!(
            /* line 42 */
            mtr.get_row(39),
            [vec![0.into(); 85], vec![1.into(); 3], vec![0.into(); 29]].concat()
        );
        assert_eq!(
            /* line 43 */
            mtr.get_row(40),
            [vec![0.into(); 88], vec![1.into(); 3], vec![0.into(); 26]].concat()
        );
        assert_eq!(
            /* line 44 */
            mtr.get_row(41),
            [vec![0.into(); 91], vec![1.into(); 3], vec![0.into(); 23]].concat()
        );
        assert_eq!(
            /* line 45 */
            mtr.get_row(42),
            [vec![0.into(); 94], vec![1.into(); 3], vec![0.into(); 20]].concat()
        );
        assert_eq!(
            /* line 46 */
            mtr.get_row(43),
            [vec![0.into(); 97], vec![1.into(); 3], vec![0.into(); 17]].concat()
        );
        assert_eq!(
            /* line 47 */
            mtr.get_row(44),
            [vec![0.into(); 100], vec![1.into(); 3], vec![0.into(); 14]].concat()
        );
        assert_eq!(
            /* line 48 */
            mtr.get_row(45),
            [vec![0.into(); 103], vec![1.into(); 3], vec![0.into(); 11]].concat()
        );
        assert_eq!(
            /* line 49 */
            mtr.get_row(46),
            [vec![0.into(); 106], vec![1.into(); 3], vec![0.into(); 8]].concat()
        );
        assert_eq!(
            /* line 50 */
            mtr.get_row(47),
            [vec![0.into(); 109], vec![1.into(); 3], vec![0.into(); 5]].concat()
        );
        assert_eq!(
            /* line 51 */
            mtr.get_row(48),
            [
                vec![0.into(); 43],
                vec![1.into(), 0.into(), 0.into(), 1.into()],
                vec![0.into(); 63],
                vec![1.into()],
                vec![0.into(); 6]
            ]
            .concat()
        );
        assert_eq!(
            /* line 52 */
            mtr.get_row(49),
            [
                vec![0.into(); 40],
                vec![1.into(), 0.into(), 0.into(), 1.into()],
                vec![0.into(); 63],
                vec![1.into()],
                vec![0.into(); 9]
            ]
            .concat()
        );
        assert_eq!(
            /* line 53 */
            mtr.get_row(50),
            [
                vec![0.into(); 37],
                vec![1.into(), 0.into(), 0.into(), 1.into()],
                vec![0.into(); 63],
                vec![1.into()],
                vec![0.into(); 12]
            ]
            .concat()
        );
        assert_eq!(
            /* line 54 */
            mtr.get_row(51),
            [
                vec![0.into(); 37],
                vec![1.into()],
                vec![0.into(); 63],
                vec![1.into()],
                vec![0.into(); 10],
                vec![1.into()],
                vec![0.into(); 4]
            ]
            .concat()
        );
        assert_eq!(
            /* line 55 */
            mtr.get_row(52),
            [
                vec![0.into(); 31],
                vec![1.into(), 0.into(), 0.into(), 1.into()],
                vec![0.into(); 63],
                vec![1.into()],
                vec![0.into(); 18]
            ]
            .concat()
        );
        assert_eq!(
            /* line 56 */
            mtr.get_row(53),
            [
                vec![0.into(); 28],
                vec![1.into(), 0.into(), 0.into(), 1.into()],
                vec![0.into(); 63],
                vec![1.into()],
                vec![0.into(); 21]
            ]
            .concat()
        );
        assert_eq!(
            /* line 57 */
            mtr.get_row(54),
            [
                vec![0.into(); 25],
                vec![1.into(), 0.into(), 0.into(), 1.into()],
                vec![0.into(); 63],
                vec![1.into()],
                vec![0.into(); 24]
            ]
            .concat()
        );
        assert_eq!(
            /* line 58 */
            mtr.get_row(55),
            [
                vec![0.into(); 25],
                vec![1.into()],
                vec![0.into(); 63],
                vec![1.into()],
                vec![0.into(); 23],
                vec![1.into()],
                vec![0.into(); 3]
            ]
            .concat()
        );
        assert_eq!(
            /* line 59 */
            mtr.get_row(56),
            [
                vec![0.into(); 19],
                vec![1.into(), 0.into(), 0.into(), 1.into()],
                vec![0.into(); 63],
                vec![1.into()],
                vec![0.into(); 30],
            ]
            .concat()
        );
        assert_eq!(
            /* line 60 */
            mtr.get_row(57),
            [
                vec![0.into(); 16],
                vec![1.into(), 0.into(), 0.into(), 1.into()],
                vec![0.into(); 63],
                vec![1.into()],
                vec![0.into(); 33],
            ]
            .concat()
        );
        assert_eq!(
            /* line 61 */
            mtr.get_row(58),
            [
                vec![0.into(); 13],
                vec![1.into(), 0.into(), 0.into(), 1.into()],
                vec![0.into(); 63],
                vec![1.into()],
                vec![0.into(); 36],
            ]
            .concat()
        );
        assert_eq!(
            /* line 62 */
            mtr.get_row(59),
            [
                vec![0.into(); 13],
                vec![1.into()],
                vec![0.into(); 63],
                vec![1.into()],
                vec![0.into(); 36],
                vec![1.into()],
                vec![0.into(); 2]
            ]
            .concat()
        );
        assert_eq!(
            /* line 63 */
            mtr.get_row(60),
            [
                vec![0.into(); 7],
                vec![1.into(), 0.into(), 0.into(), 1.into()],
                vec![0.into(); 63],
                vec![1.into()],
                vec![0.into(); 42]
            ]
            .concat()
        );
        assert_eq!(
            /* line 64 */
            mtr.get_row(61),
            [
                vec![0.into(); 4],
                vec![1.into(), 0.into(), 0.into(), 1.into()],
                vec![0.into(); 63],
                vec![1.into()],
                vec![0.into(); 45]
            ]
            .concat()
        );
        assert_eq!(
            /* line 65 */
            mtr.get_row(62),
            [
                vec![0.into(); 1],
                vec![1.into(), 0.into(), 0.into(), 1.into()],
                vec![0.into(); 63],
                vec![1.into()],
                vec![0.into(); 48]
            ]
            .concat()
        );
        assert_eq!(
            /* line 66 */
            mtr.get_row(63),
            [
                vec![0.into(); 1],
                vec![1.into()],
                vec![0.into(); 63],
                vec![1.into()],
                vec![0.into(); 49],
                vec![1.into(), 1.into()],
            ]
            .concat()
        );
    }
}
