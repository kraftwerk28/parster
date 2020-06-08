use regex::Regex;
use std::{
    collections::{BTreeMap, LinkedList},
    env, fmt, fs,
    time::Instant,
};

pub enum JSONValue {
    Number(f64),
    String(String),
    Bool(bool),
    Array(LinkedList<JSONValue>),
    Object(BTreeMap<String, JSONValue>),
    Null,
}

impl JSONValue {
    fn repr(&self, indent: usize) -> String {
        let indstr = (0..=indent).map(|_| "  ").collect::<String>();
        let r = match self {
            JSONValue::Null => String::from("null"),
            JSONValue::String(s) => {
                let repr = format!("\"{}\"", s.as_str());
                repr
            }
            JSONValue::Object(map) => {
                if map.len() == 0 {
                    return "{}".to_string();
                }
                let indstrend = (0..indent).map(|_| "  ").collect::<String>();
                let repr = map
                    .iter()
                    .map(|(key, val)| {
                        format!(
                            "{}\"{}\": {}",
                            indstr,
                            key,
                            val.repr(indent + 1)
                        )
                    })
                    .collect::<Vec<String>>()
                    .join(",\n");
                format!("{{\n{}\n{}}}", repr, indstrend)
            }
            JSONValue::Array(arr) => {
                if arr.len() == 0 {
                    return "[]".to_string();
                }
                let indstrend = (0..indent).map(|_| "  ").collect::<String>();
                let repr = arr
                    .iter()
                    .map(|val| format!("{}{}", indstr, val.repr(indent + 1)))
                    .collect::<Vec<String>>()
                    .join(",\n");
                format!("[\n{}\n{}]", repr, indstrend)
            }
            JSONValue::Number(n) => n.to_string(),
            JSONValue::Bool(n) => n.to_string(),
        };
        r
    }

    //     pub fn from_path(&self, path: &str) -> &Self {
    //         let re = Regex::new(r".(\w+)|\[(\d+)\]").unwrap();
    //         for c in re.captures_iter(path) {
    //             println!("{:?}", c.get(1).unwrap().as_str());
    //         }
    //         match self {
    //             JSONValue::Object(map) => self,
    //             JSONValue::Array(arr) => self,
    //             _ => self,
    //         }
    //     }

    //     fn get_by_key(&self, path: &str) -> Option<&Self> {
    //         match self {
    //             JSONValue::Object(map) => Some(self),
    //             JSONValue::Array(arr) => {
    //                 if let Ok(n) = path.parse::<usize>() {
    //                     Some(arr.iter().nth(n).unwrap())
    //                 } else {
    //                     None
    //                 }
    //             }
    //             _ => None,
    //         }
    //     }
}

impl fmt::Display for JSONValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let repr = self.repr(0);
        write!(f, "{}", repr)
    }
}

const FALSE: &str = "alse";
const TRUE: &str = "rue";
const NULL: &str = "ull";

struct JSON<'a> {
    iter: std::str::Chars<'a>,
    cur_tok: char,
    pos: usize,
}

impl<'a> JSON<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut iter = input.chars();
        let cur_tok = iter.next().unwrap();
        Self {
            iter,
            cur_tok,
            pos: 0,
        }
    }

    fn next_tok(&mut self) -> char {
        let t = self.iter.next().unwrap_or('\0');
        self.pos += 1;
        if t.is_ascii_whitespace() {
            return self.next_tok();
        }
        self.cur_tok = t;
        t
    }

    fn expect(&mut self, ch: char) -> () {
        if self.cur_tok != ch {
            format!("Unexpected token {}. Expected {}.", self.cur_tok, ch);
        }
        self.next_tok();
    }

    fn parse_any(&mut self) -> JSONValue {
        match self.cur_tok {
            '{' => self.parse_object(),
            '[' => self.parse_array(),
            't' | 'f' | 'n' => self.parse_literal(),
            '"' => self.parse_string(),
            '0'..='9' | '-' => self.parse_number(),
            c => panic!("Unexpected token \"{}\" at start of JSON value.", c),
        }
    }

    fn parse_object(&mut self) -> JSONValue {
        let mut map = BTreeMap::new();
        self.expect('{');
        if self.cur_tok == '}' {
            self.next_tok();
            return JSONValue::Object(map);
        }
        loop {
            let key = match self.parse_string() {
                JSONValue::String(s) => s,
                _ => "".to_string(),
            };
            self.expect(':');
            let value = self.parse_any();
            map.insert(key, value);
            let c = self.cur_tok;
            self.next_tok();
            match c {
                ',' => continue,
                '}' => break,
                _ => panic!("Unexpected token \"{}\" in the end of object.", c),
            }
        }
        JSONValue::Object(map)
    }

    fn parse_array(&mut self) -> JSONValue {
        let mut arr = LinkedList::new();
        self.expect('[');
        if self.cur_tok == ']' {
            self.next_tok();
            return JSONValue::Array(arr);
        }
        loop {
            let value = self.parse_any();
            arr.push_back(value);
            let c = self.cur_tok;
            self.next_tok();
            match c {
                ',' => continue,
                ']' => break,
                _ => panic!("Unexpected token \"{}\" in the end of array.", c),
            }
        }
        JSONValue::Array(arr)
    }

    fn parse_number(&mut self) -> JSONValue {
        let mut s = self.cur_tok.to_string();
        loop {
            match self.next_tok() {
                t if t.is_ascii_digit() || t == '.' => {
                    s.push(t);
                }
                _ => break,
            }
        }
        JSONValue::Number(s.parse::<f64>().unwrap())
    }

    fn parse_string(&mut self) -> JSONValue {
        let striter = self.iter.by_ref().take_while(|&c| c != '"');
        let s = striter.collect();
        self.next_tok();
        JSONValue::String(s)
    }

    fn parse_literal(&mut self) -> JSONValue {
        let biter = self.iter.by_ref();
        match self.cur_tok {
            't' => {
                assert!(biter.take(TRUE.len()).eq(TRUE.chars()));
                self.next_tok();
                JSONValue::Bool(true)
            }
            'f' => {
                assert!(biter.take(FALSE.len()).eq(FALSE.chars()));
                self.next_tok();
                JSONValue::Bool(false)
            }
            'n' => {
                assert!(biter.take(NULL.len()).eq(NULL.chars()));
                self.next_tok();
                JSONValue::Null
            }
            _ => panic!("Unexpected literal."),
        }
    }

    pub fn parse(&mut self) -> JSONValue {
        self.parse_any()
    }
}

// struct Test {
//     foo: i32,
//     bar: String,
// }

// macro_rules! mac {
//     ($x: ty, $($key: ident, $val: expr), *) => {
//         $x {
//             $($key: $val, )*
//         }
//     };
// }

fn main() {
    let fname = env::args().nth(1).expect("Must pass filename to parse.");
    let input = fs::read_to_string(fname).unwrap();
    let t = Instant::now();
    let p = JSON::new(input.as_str()).parse();
    println!("time: {}ms", t.elapsed().as_micros() as f64 / 1000f64);
    // p.from_path("asdf");
    if let JSONValue::Array(list) = &p {
        println!("Len: {}", list.len());
    }
    if let JSONValue::Object(map) = &p {
        let deps = map.get("dependencies").unwrap();
        if let JSONValue::Object(map_) = &deps {
            println!("Map len: {}", map_.len());
        }
    }
}
