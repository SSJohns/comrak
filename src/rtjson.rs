use ctype::isspace;
use nodes::{TableAlignment, NodeValue, ListType, AstNode};
use parser::ComrakOptions;

/// Formats an AST as HTML, modified by the given options.
pub fn format_document<'a>(root: &'a AstNode<'a>, options: &ComrakOptions) -> String {
    let mut f = RTJsonFormatter::new(options);
    /*f.s += "'document' : [";*/
    f.format(root, false);
    /*f.s += "]";*/
    f.s
}

struct RTJsonFormatter<'o> {
    s: String,
    f: Vec<[u8; 3]>,
    link: bool,
    v: Vec<[u8; 3]>,
    zero_index: usize,
    options: &'o ComrakOptions,
}

fn tagfilter(literal: &str) -> bool {
    lazy_static! {
        static ref TAGFILTER_BLACKLIST: [&'static str; 9] =
            ["title", "textarea", "style", "xmp", "iframe",
             "noembed", "noframes", "script", "plaintext"];
    }

    if literal.len() < 3 || literal.as_bytes()[0] != b'<' {
        return false;
    }

    let mut i = 1;
    if literal.as_bytes()[i] == b'/' {
        i += 1;
    }

    for t in TAGFILTER_BLACKLIST.iter() {
        if literal[i..].to_string().to_lowercase().starts_with(t) {
            let j = i + t.len();
            return isspace(literal.as_bytes()[j]) || literal.as_bytes()[j] == b'>' ||
                   (literal.as_bytes()[j] == b'/' && literal.len() >= j + 2 &&
                    literal.as_bytes()[j + 1] == b'>');
        }
    }

    false
}

fn tagfilter_block(input: &str, mut o: &mut String) {
    let src = input.as_bytes();
    let size = src.len();
    let mut i = 0;

    while i < size {
        let org = i;
        while i < size && src[i] != b'<' {
            i += 1;
        }

        if i > org {
            *o += &input[org..i];
        }

        if i >= size {
            break;
        }

        if tagfilter(&input[i..]) {
            *o += "&lt;";
        } else {
            o.push('<');
        }

        i += 1;
    }
}

impl<'o> RTJsonFormatter<'o> {
    fn new(options: &'o ComrakOptions) -> Self {
        RTJsonFormatter {
            s: String::with_capacity(1024),
            f: Vec::new(),
            v: Vec::new(),
            link: false,
            zero_index: 0,
            options: options,
        }
    }

    fn cr(&mut self) {
        /*let l = self.s.len();
        if l > 0 && self.s.as_bytes()[l - 1] != b'\n' {
            self.s += "\n";
        }*/
    }

    fn escape(&mut self, buffer: &str) {
        lazy_static! {
            static ref NEEDS_ESCAPED: [bool; 256] = {
                let mut sc = [false; 256];
                for &c in &['"', '&', '<', '>', '\''] {
                    sc[c as usize] = true;
                }
                sc
            };
        }

        let src = buffer.as_bytes();
        let size = src.len();
        let mut i = 0;

        while i < size {
            let org = i;
            while i < size && !NEEDS_ESCAPED[src[i] as usize] {
                i += 1;
            }

            if i > org {
                self.s += &buffer[org..i];
            }

            if i >= size {
                break;
            }

            match src[i] as char {
                '"' => self.s += "&quot;",
                '&' => self.s += "&amp;",
                '<' => self.s += "&lt;",
                '>' => self.s += "&gt;",
                '\'' => self.s += "&#27;",
                _ => unreachable!(),
            }

            i += 1;
        }
    }

    fn escape_href(&mut self, buffer: &str) {
        lazy_static! {
            static ref HREF_SAFE: [bool; 256] = {
                let mut a = [false; 256];
                for &c in b"-_.+!*'(),%#@?=;:/,+&$abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".iter() {
                    a[c as usize] = true;
                }
                a
            };
        }

        let src = buffer.as_bytes();
        let size = src.len();
        let mut i = 0;

        while i < size {
            let org = i;
            while i < size && HREF_SAFE[src[i] as usize] {
                i += 1;
            }

            if i > org {
                self.s += &buffer[org..i];
            }

            if i >= size {
                break;
            }

            match src[i] as char {
                '&' => self.s += "&amp;",
                '\'' => self.s += "&#x27;",
                _ => self.s += &format!("%{:02X}", src[i]),
            }

            i += 1;
        }
    }

    fn pull_formatting(&mut self) {
        for a in self.f.iter() {
            self.s += &format!("{:?},", a);
        }
        self.zero_index = 0;
        self.f.clear();
        self.v.clear();
    }
    //
    // fn split_on_char(&mut self, seperators: &[char], literal: String){
    //     let items = literal.split(seperators);
    //     for item in it {
    //         self.s += "{'e':'raw','t':'";
    //         println!("{}", item);
    //         self.escape(item);
    //         self.s += "'},";
    //     }
    // }

    fn format_children<'a>(&mut self, node: &'a AstNode<'a>, plain: bool) {
        for n in node.children() {
            self.format(n, plain);
        }
    }

    fn format<'a>(&mut self, node: &'a AstNode<'a>, plain: bool) {
        if plain {
            match node.data.borrow().value {
                NodeValue::Text(ref literal) |
                NodeValue::Code(ref literal) |
                NodeValue::HtmlInline(ref literal) => self.escape(literal),
                NodeValue::LineBreak | NodeValue::SoftBreak => self.s += " { 'e': 'br' },",
                _ => (),
            }
            self.format_children(node, true);
        } else {
            let new_plain = self.format_node(node, true);
            self.format_children(node, new_plain);
            self.format_node(node, false);
        }
    }

    fn format_node<'a>(&mut self, node: &'a AstNode<'a>, entering: bool) -> bool {
        match node.data.borrow().value {
            NodeValue::Document => {
              if entering {
                self.s += "'document':[";
              } else {
                self.s += "],";
              }
              },
            NodeValue::BlockQuote => {
                if entering {
                    self.cr();
                    self.s += "{ 'e': 'blockquote', 'c': [  ";
                } else {
                    self.cr();
                    self.s += " ], },";
                }
            }
            NodeValue::List(ref nl) => {
                if entering {
                    self.cr();
                    if nl.list_type == ListType::Bullet {
                        self.s += "{ 'e': 'list', 'o': False, 'c': [";
                    } else {
                        self.s += "{ 'e': 'list', 'o': True, 'c': [";
                    } /*else {
                        self.s += &format!("<ol start=\"{}\">\n", nl.start);
                    }*/
                } else {
                    self.s += "], },";
                }
            }
            NodeValue::Item(..) => {
                if entering {
                    self.cr();
                    self.s += "{ 'e': 'li', 'c': [";
                } else {
                    self.s += "], }, ";
                }
            }
            NodeValue::Heading(ref nch) => {
                if entering {
                    self.cr();
                    self.s += &format!("{{ 'e': 'h', 'l': {}, 'c': [", nch.level);
                } else {
                    self.s += "], },";
                }
            }
            NodeValue::CodeBlock(ref ncb) => {
                if entering {
                    self.cr();

                    if ncb.info.is_empty() {
                        self.s += "{'e':'code','c':[";
                    } else {
                        let mut first_tag = 0;
                        while first_tag < ncb.info.len() &&
                              !isspace(ncb.info.as_bytes()[first_tag]) {
                            first_tag += 1;
                        }

                        self.s += "{'e':'code','l':'";
                        self.escape(&ncb.info[..first_tag]);
                        self.s += "','c':[";
                    }

                    for it in ncb.literal.split('\n') {
                        self.s += "{'e':'raw','t':'";
                        println!("item:{}", it);
                        self.escape(it);
                        self.s += "'},";
                    }
                    //self.escape(&ncb.literal);
                    self.s += "],},";
                }
            }
            NodeValue::HtmlBlock(ref nhb) => {
                if entering {
                    self.cr();
                    if self.options.ext_tagfilter {
                        tagfilter_block(&nhb.literal, &mut self.s);
                    } else {
                        self.s += &nhb.literal;
                    }
                    self.cr();
                }
            }
            NodeValue::ThematicBreak => {
                if entering {
                    self.cr();
                    self.s += "{ e: 'br' }";
                }
            }
            NodeValue::Paragraph => {
                let tight = match node.parent()
                          .and_then(|n| n.parent())
                          .map(|n| n.data.borrow().value.clone()) {
                    Some(NodeValue::List(nl)) => nl.tight,
                    _ => false,
                };

                if entering {
                    if !tight {
                        self.cr();
                        self.s += "{ 'e': 'par', 'c': [";
                    }
                } else if !tight {
                    self.s += "], },";
                }
            }
            NodeValue::Text(ref literal) => {
                if entering {
                    match node.parent().unwrap().data.borrow().value {
                        NodeValue::Link(ref nl) => self.escape(literal),
                        NodeValue::Text(ref literal) |
                        NodeValue::Code(ref literal) |
                        NodeValue::HtmlInline(ref literal) => self.escape(literal),
                        NodeValue::Strong | NodeValue::Strikethrough | NodeValue::Emph => self.escape(literal),
                        NodeValue::LineBreak | NodeValue::SoftBreak => self.s += " { 'e': 'br' },",
                        _ => {
                            //println!("Possibly first child of bold {:?}", node.parent().unwrap().data.borrow().value);
                            if node.same_node(node.parent().unwrap().first_child().unwrap()) {
                                if self.zero_index == 0 {
                                    self.zero_index = self.s.len();
                                    println!("IN text: {:?}, {}", self.s.len(), self.zero_index);
                                }
                                self.s += "{ 'e': 'text', 't' : '";
                                self.escape(literal);
                            }
                        },
                    }
                    // if self.link {
                    //     self.escape(literal);
                    // } else if node.same_node(node.parent().unwrap().first_child().unwrap() ){
                    //     println!("Possibly first child of bold {:?}", node.parent().unwrap().data.borrow().value);
                    //     if self.zero_index == 0 {
                    //         self.zero_index = self.s.len();
                    //         println!("{:?}, {}", self.s.len(), self.zero_index);
                    //     }
                    //     self.s += "{ 'e': 'text', 't' : '";
                    //     self.escape(literal);
                    // }
                } else {
                    println!("Ending is picking? {:?}", node.parent().unwrap().data.borrow().value);
                    match node.parent().unwrap().data.borrow().value {
                        NodeValue::Link(ref nl) => (),
                        NodeValue::Code(ref literal) => (),
                        NodeValue::Strikethrough |
                        NodeValue::Superscript |
                        NodeValue::Strong | NodeValue::Emph => (),
                        _ => {
                            if node.same_node(node.parent().unwrap().last_child().unwrap()) {
                                if !self.f.is_empty() {
                                    // let siz = self.s.len()- self.zero_index;
                                    //self.f.push(siz);
                                    //let form = format!("[{:?}]", self.f );
                                    //self.f.clear();
                                    self.s += "', 'f': ";
                                    self.pull_formatting();
                                    self.s += "},";
                                } else {
                                    self.s += "' },";
                                }
                            } else {
                                ()
                            }
                        },
                    }
                    // println!("{:?}", node.parent().unwrap().data.borrow().value);
                    // let start = node.parent().unwrap().data.borrow().value;
                    // if start != NodeValue::Strong || start != NodeValue::Emph {
                    //     if !self.f.is_empty() {
                    //         // let siz = self.s.len()- self.zero_index;
                    //         //self.f.push(siz);
                    //         //let form = format!("[{:?}]", self.f );
                    //         //self.f.clear();
                    //         self.s += "', 'f': ";
                    //         self.pull_formatting();
                    //         self.s += "},";
                    //     } else {
                    //         self.s += "' },";
                    //     }
                    // } else {
                    //     self.escape(literal);
                    // }
                }
            }
            NodeValue::LineBreak => {
                if entering {
                    self.s += "{ 'e': 'br' },";
                }
            }
            NodeValue::SoftBreak => {
                if entering {
                    if self.options.hardbreaks {
                        self.s += "{ 'e': 'br' },";
                    } else {
                        self.s += "\n";
                    }
                }
            }
            NodeValue::Code(ref literal) => {
                if entering {
                    /*self.s += "<strong>";*/
                    // self.l.push(self.s.len());
                    let val = 64;
                    // if !self.v.is_empty() {
                    //   for a in self.v.iter() {
                    //       val += a[0];
                    //   }
                    // }
                    let v: [u8; 3] = [val, (self.s.len() - self.zero_index) as u8, 0];
                    //self.f.push(v);
                    self.v.push(v);
                    self.escape(literal);
                } else {
                    let vain = &mut self.v.pop().unwrap();
                    let siz = self.s.len() - self.zero_index - vain[1] as usize;
                    vain[2] = siz as u8;
                    //self.f. = (self.s.len() as u8) - self.l;
                    self.f.push(*vain);
                    //self.v.pop();
                    /*self.s += "</strong>";*/
                }
            }
            NodeValue::HtmlInline(ref literal) => {
                if entering {
                    if self.options.ext_tagfilter && tagfilter(literal) {
                        self.s += "&lt;";
                        self.s += &literal[1..];
                    } else {
                        self.s += literal;
                    }
                }
            }
            NodeValue::Strong => {
                if entering {
                    /*self.s += "<strong>";*/
                    // self.l.push(self.s.len());
                    let mut val = 1;
                    if !self.v.is_empty() {
                      for a in self.v.iter() {
                          val += a[0];
                      }
                    }
                    let v = [val, (self.s.len() - self.zero_index) as u8, 0];
                    //self.f.push(v);
                    self.v.push(v);
                    println!("bold {:?}", self.f);
                } else {
                    println!("bold {:?}", self.s);
                    let vain = &mut self.v.pop().unwrap();
                    println!("vainity {:?}", vain);
                    let siz = self.s.len()  - self.zero_index - vain[1] as usize;
                    vain[2] = siz as u8;
                    //self.f. = (self.s.len() as u8) - self.l;
                    self.f.push(*vain);
                    /*self.s += "</strong>";*/
                }
            }
            NodeValue::Emph => {
                if entering {
                    /*self.s += "<strong>";*/
                    // self.l.push(self.s.len());
                    let mut val = 2;
                    if !self.v.is_empty() {
                      for a in self.v.iter() {
                          val += a[0];
                      }
                    }
                    let v = [val, (self.s.len() - self.zero_index) as u8, 0];
                    //self.f.push(v);
                    self.v.push(v);
                } else {
                    let vain = &mut self.v.pop().unwrap();
                    let siz = self.s.len()  - self.zero_index - vain[1] as usize;
                    vain[2] = siz as u8;
                    //self.f. = (self.s.len() as u8) - self.l;
                    self.f.push(*vain);
                    //self.v.pop();
                    /*self.s += "</strong>";*/
                }
            }
            NodeValue::Strikethrough => {
                if entering {
                    /*self.s += "<strong>";*/
                    // self.l.push(self.s.len());
                    let mut val = 8;
                    if !self.v.is_empty() {
                      for a in self.v.iter() {
                          val += a[0];
                      }
                    }
                    let v = [val, (self.s.len() - self.zero_index) as u8,0];
                    //self.f.push(v);
                    self.v.push(v);
                } else {
                    let vain = &mut self.v.pop().unwrap();
                    let siz = self.s.len()  - self.zero_index - vain[1] as usize;
                    vain[2] = siz as u8;
                    //self.f. = (self.s.len() as u8) - self.l;
                    self.f.push(*vain);
                    //self.v.pop();
                    /*self.s += "</strong>";*/
                }
            }
            NodeValue::Superscript => {
                if entering {
                    /*self.s += "<strong>";*/
                    let mut val = 32;
                    if !self.v.is_empty() {
                      for a in self.v.iter() {
                          val += a[0];
                      }
                    }
                    let v = [val, (self.s.len() - self.zero_index) as u8, 0];
                    //self.f.push(v);
                    self.v.push(v);
                } else {
                    let vain = &mut self.v.pop().unwrap();
                    let siz = self.s.len()  - self.zero_index - vain[1] as usize;
                    vain[2] = siz as u8;
                    //self.f. = (self.s.len() as u8) - self.l;
                    self.f.push(*vain);
                    //self.v.pop();
                    /*self.s += "</strong>";*/
                }
            }
            NodeValue::Link(ref nl) => {
                if entering {
                    self.s += "{'e':'link','u':'";
                    self.escape_href(&nl.url);
                    self.link = true;
                    /*if !nl.title.is_empty() {
                        self.s += "\" title=\"";
                        self.escape(&nl.title);
                    }*/
                    self.s += "','t':'";
                    self.zero_index = self.s.len();
                } else {
                    self.link = false;
                    self.s += "', 'f': [";
                    self.pull_formatting();
                    self.s += "],},";
                }
            }
            NodeValue::Image(ref nl) => {
                if entering {
                    self.s += "<img src=\"";
                    self.escape_href(&nl.url);
                    self.s += "\" alt=\"";
                    return true;
                } else {
                    if !nl.title.is_empty() {
                        self.s += "\" title=\"";
                        self.escape(&nl.title);
                    }
                    self.s += "\" />";
                }
            }
            NodeValue::Table(..) => {
                if entering {
                    self.cr();
                    self.s += "{ 'e': 'table', ";
                } else {
                    if !node.last_child()
                            .unwrap()
                            .same_node(node.first_child().unwrap()) {
                        self.s += "],";
                    }
                    self.s += "},";
                }
            }
            NodeValue::TableRow(header) => {
                if entering {
                    self.cr();
                    if header {
                        self.s += "'h': [";
                        self.cr();
                    }
                    self.s += "[ ";
                } else {
                    self.cr();
                    self.s += "],";
                    if header {
                        self.cr();
                        self.s += "],";
                        self.cr();
                        self.s += "'b': [";
                    }
                }
            }
            NodeValue::TableCell => {
                let row = &node.parent().unwrap().data.borrow().value;
                let in_header = match *row {
                    NodeValue::TableRow(header) => header,
                    _ => panic!(),
                };

                let table = &node.parent()
                                 .unwrap()
                                 .parent()
                                 .unwrap()
                                 .data
                                 .borrow()
                                 .value;
                let alignments = match *table {
                    NodeValue::Table(ref alignments) => alignments,
                    _ => panic!(),
                };

                if entering {
                    self.cr();
                    if in_header {
                        self.s += "{ ";
                    } else {
                        self.s += "{ 'c': [ ";
                    }

                    let mut start = node.parent().unwrap().first_child().unwrap();
                    let mut i = 0;
                    while !start.same_node(node) {
                        i += 1;
                        start = start.next_sibling().unwrap();
                    }

                    match alignments[i] {
                        TableAlignment::Left => self.s += " 'a': 'l',",
                        TableAlignment::Right => self.s += " 'a': 'r',",
                        TableAlignment::Center => self.s += " 'a': 'c',",
                        TableAlignment::None => (),
                    }

                    self.s += "'c': [ ";
                } else if in_header {
                    self.s += " ],";
                } else {
                    self.s += " ],";
                }
            }
        }
        false
    }
}
