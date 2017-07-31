use {Arena, parse_document, ComrakOptions};
use cm;
use html;
use rtjson;

fn compare_strs(output: &str, expected: &str, kind: &str) {
    if output != expected {
        println!("Running {} test", kind);
        println!("Got:");
        println!("==============================");
        println!("{}", output);
        println!("==============================");
        println!();
        println!("Expected:");
        println!("==============================");
        println!("{}", expected);
        println!("==============================");
        println!();
    }
    assert_eq!(output, expected);
}

fn rtjson(input: &str, expected: &str) {
    rtjson_opts(input, expected, |_| ());
}

fn rtjson_opts<F>(input: &str, expected: &str, opts: F)
where
    F: Fn(&mut ComrakOptions),
{
    let arena = Arena::new();
    let mut options = ComrakOptions::default();
    opts(&mut options);

    let root = parse_document(&arena, &input.chars().collect::<String>(), &options);
    let output = rtjson::format_document(root, &options);
    compare_strs(&output, expected, "regular");

    let md = cm::format_document(root, &options);
    let root = parse_document(&arena, &md.chars().collect::<String>(), &options);
    let output_from_rt = rtjson::format_document(root, &options);
    compare_strs(&output_from_rt, expected, "roundtrip");
}

// #[test]
// fn basic() {
//     rtjson(
//         concat!(
//             "[this is a link with **bold** and *italic*](https://reddit.com)"
//         ),
//         concat!(
//             "'document': [ { 'e': 'par', 'c': [{ 'e': 'link', 'u': 'https://reddit.com', 't': 'this is a link with bold and italic', 'f': [[1, 20, 4], [2, 29, 6]], }, ], }, ],"
//         ),
//     );
// }
//
// #[test]
// fn basic_2() {
//     rtjson(
//         concat!(
//             "*Hello Reddit*, this an example paragraph. Read more RTJson [here](https://docs.google.com/document/d/1Qpf2tl8iabZIEKvUSE3bFRV2QCIjxaXn-6Mblvojpvs/edit#heading=h.w2llmo96i6e1)",
//         ),
//         concat!(
//             "'document': [",
//                 "{",
//                     "'e': 'par',",
//                     "'c': [",
//                         "{",
//                             "'e': 'text',",
//                             "'t': 'Hello Reddit, this an example paragraph. Read more RTJson ',",
//                             "'f': [[2, 0, 12]],",
//                         "},",
//                         "{",
//                             "'e': 'link',",
//                             "'t': 'here',",
//                             "'u': 'https://docs.google.com/document/d/1Qpf2tl8iabZIEKvUSE3bFRV2QCIjxaXn-6Mblvojpvs/edit#heading=h.w2llmo96i6e1',",
//                         "}",
//                     "]",
//                 "},",
//             "]",
//         ),
//     );
// }
//
// #[test]
// fn heading() {
//     rtjson(
//         concat!(
//             "### This heading contains plain text, [a link](https://reddit.com), and a u/username.\n\nHello, this is a paragraph."
//         ),
//         concat!(
//             "'document':[{'e': 'h','l': 3,'c': [{'e': 'raw','t': 'This heading contains plain text, '},{'e': 'link','u': 'https://reddit.com','t': 'a link'},{'e': 'raw','t': ', and a '},{'e': 'u/','t': 'username'},{'e': 'raw','t': '.'}],},{'e': 'par','c': [{'e': 'text','t': 'Hello, this is a paragraph.'}],},]"
//         ),
//     );
// }
//
// #[test]
// fn blockquote_1() {
//     rtjson(
//         concat!(
//             ">This post begins with a blockquote.\n\nThis post has a paragraph in the middle.\n\n>This post ends with a blockquote."
//         ),
//         concat!(
//             "'document': [{'e': 'blockquote','c': [{'e': 'par','c': [{'e': 'text','t': 'This post begins with a blockquote.',}]}]},{'e': 'par','c': [{'e': 'text','t': 'This post has a paragraph in the middle.',}]},{'e': 'blockquote','c': [{'e': 'par','c': [{'e': 'text','t': 'This post ends with a blockquote.',}]}]}]"
//         ),
//     );
// }
//
// #[test]
// fn blockquote_only() {
//     rtjson(
//         concat!(
//             ">A blockquote with nothing else."
//         ),
//         concat!(
//             "'document': [{'e': 'blockquote','c': [{'e': 'par','c': [{'e': 'text','t': 'A blockquote with nothing else.'}]}],}]"
//         ),
//     );
// }
//
// #[test]
// fn blockquote_with_br() {
//     rtjson(
//         concat!(
//             ">Line proceeding; this line has a [link](https://reddit.com) and a r/redditlink  \n>Line preceding; no line proceeding  \n>  \n>No line preceding; no line proceeding  \n>  \n>No line preceding; line proceeding  \n>Line preceding"
//         ),
//         concat!(
//             "'document': [{'e': 'blockquote','c': [{'e': 'par','c': [{'e': 'text','t': 'Line proceeding; this line has a '},{'e': 'link','u': 'https://reddit.com','t': 'link'},{'e': 'text','t': ' and a '},{'e': 'r/','t': 'redditlink'}]},{'e': 'par','c': [{'e': 'text','t': 'Line preceding; no line proceeding'}]},{'e': 'par','c': [{'e': 'text','t': ''}]},{'e': 'par','c': [{'e': 'text','t': 'No line preceding; no line proceeding'}]},{'e': 'par','c': [{'e': 'text','t': ''}]},{'e': 'par','c': [{'e': 'text','t': 'No line preceding; line proceeding'}]},{'e': 'par','c': [{'e': 'text','t': 'Line preceding'}]}]}]"
//         ),
//     );
// }
//
// #[test]
// fn blockquote_newline_literal() {
//     rtjson(
//         concat!(
//             ">This post ends with a blockquote\n\nwith embedded newlines."
//         ),
//         concat!(
//             "'document': [{'e': 'blockquote','c': [{'e': 'par','c': [{'e': 'text','t': 'This post ends with a blockquote\n\nwith embedded newlines.',}]}]}]"
//         ),
//     );
// }
//
// #[test]
// fn redditlink() {
//     rtjson(
//         concat!(
//             "Hello, **this is bold**, *this is italic*, ***this is both***. And this is a u/username and a r/subreddit."
//         ),
//         concat!(
//             "'document': [{'e': 'par','c': [{'e': 'text','t': 'Hello, this is bold, this is italic, this is both. And this is a ', 'f': [[1, 7, 12], [2, 21, 14], [3, 37, 12]],},{'e': 'u/','t': 'username'},{'e': 'text','t': ' and a '},{'e': 'r/','t': 'subreddit'},{'e': 'text','t': '.'}]}]"
//         ),
//     );
// }
//
// #[test]
// fn ul() {
//     rtjson(
//         concat!(
//             "Below this is a list:\n\n* First item\n* Second item\n* Third item\n\nAbove this is a list."
//         ),
//         concat!(
//             "'document': [{'e': 'par','c': [{'e': 'text','t': 'Below this is a list:'}]},{'e': 'list','o': False,'c': [{'e': 'li','c': [{'e': 'text','t': 'First item'}]},{'e': 'li','c': [{'e': 'text','t': 'Second item'}]},{'e': 'li','c': [{'e': 'text','t': 'Third item'}]},]},{'e': 'par','c': [{'e': 'text','t': 'Above this is a list.'}]}]"
//         ),
//     );
// }
//
// #[test]
// fn mixed_list_1() {
//     rtjson(
//         concat!(
//             "* First item\n* Second item with [a link with **bold** and *italic*](https://reddit.com) and a u/username\n   1. First item\n   2. Second item\n      * First item\n      * Second item"
//         ),
//         concat!(
//             "'document': [{'e': 'list','o': False,'c': [{'e': 'li','c': [{'e': 'text','t': 'First item'}]},{'e': 'li','c': [{'e': 'text','t': 'Second item with '},{'e': 'link','u': 'https://reddit.com','t': 'a link with bold and italic','f': [[1, 12, 4], [2, 21, 6]],},{'e': 'text','t': ' and a '},{'e': 'u/','t': 'username'}]},{'e': 'list','o': True,'c': [{'e': 'li','c': [{'e': 'text','t': 'First item'}]},{'e': 'li','c': [{'e': 'text','t': 'Second item'}]},{'e': 'list','o': False,'c': [{'e': 'li','c': [{'e': 'text','t': 'First item'}]},{'e': 'li','c': [{'e': 'text','t': 'Second item'}]}]}]}]}]"
//         ),
//     );
// }
//
#[test]
fn table_1() {
    rtjson_opts(
        concat!(
            "|Col 1|Col 2|Col 3|\n|:-|:-:|-:|\n| |**bold*****bold+italic****italic*| |"
        ),
        concat!(
            "'document': [{'e': 'table','h': [{'c': [{'e': 'text','t': 'Col 1',},]},{'a': 'c','c': [{'e': 'text','t': 'Col 2',}]},{'a': 'r','c': [{'e': 'text','t': 'Col 3',}]},],'b': [[{'c': [{'e': 'text','t': 'c1:r1',}]},{'c': [{'e': 'text','t': 'c2:r1',},]},{'c': [{'e': 'text','t': 'c3:r1',},]},],[{'c': [{'e': 'text','t': 'c1:r2',},],},{'c': [{'e': 'text','t': 'c2:r2',},],},{'c': [{'e': 'text','t': 'c3:r2',},]},],[{'c': [{'e': 'text','t': ' '}]},{'c': [{'e': 'text','t': 'c2:r3'}]},{'c': [{'e': 'text','t': 'c3:r3'}]},],[{'c': [{'e': 'text','t': 'c1:r4'}]},{'c': [{'e': 'text','t': ' '},]},{'c': [{'e': 'text','t': 'c3:r4'}]},],[{'c': [{'e': 'text','t': ' '}]},{'c': [{'e': 'text','t': 'boldbold+italicitalic','f': [[1, 0, 4], [3, 4, 11], [2, 15, 6]]}]},{'c': [{'e': 'text','t': ' '}]},],],}]"
        ),
        |opts| opts.ext_table = true,
    );
}

// #[test]
// fn table_2() {
//     rtjson_opts(
//         concat!(
//             "These are two tables:\n\n|Table|1|\n|:-|:-|\n|c1:r1|c2:r1|\n\n|Table|2|\n|:-|:-|\n|c1:r2|c2:r2|"
//         ),
//         concat!(
//             "'document': [{'e': 'par','c': [{'e': 'text','t': 'These are two tables:',}]},{'e': 'table','h': [{'c': [{'e': 'text','t': 'Table'}]},{'c': [{'e': 'text','t': '1',}]},],'b': [[{'c': [{'e': 'text','t': 'c1:r1'}]},{'c': [{'e': 'text','t': 'c2:r1'}]},]]},{'e': 'table','h': [{'c': [{'e': 'text','t': 'Table'}]},{'c': [{'e': 'text','t': '2'}]},],'b': [[{'c': [{'e': 'text','t': 'c1:r2'}]},{'c': [{'e': 'text','t': 'c2:r2'}]},]]},]"
//         ),
//         |opts| opts.ext_table = true,
//     );
// }

// #[test]
// fn special_characters() {
//     rtjson(
//         concat!(
//             "Hello reddit, \*\***this should be bold,**\*\* the stars around it should not be."
//         ),
//         concat!(
//             "'document': [{'e': 'par','c': [{'e': 'text','t': 'Hello reddit, **this should be bold,** the stars around it should not be.','f': [[1, 16, 20]]}]}]"
//         ),
//     );
// }
//
// #[test]
// fn special_characters_2() {
//     rtjson(
//         concat!(
//             "Hello reddit, \\\\*\\\\***this should be bold,**\\\\*\\\\* the stars around it should not be.\n\n\\\\> This is text with an arrow in front\n\n>This is a quote\n\n*Here we have something in italics*\n\n\\\\*Here we have something with single-stars around it\\\\*\n\n\\\\`Is this a codeblock?\\\\`\n\n\\\\~\\\\~This should not be strike through\\\\~\\\\~\n\n~~But this should be~~\n\n\\\\[Finally here we have no link\\\\]\\\\(www.example.com\\\\)\n\nwww.thisisalink.com"
//         ),
//         concat!(
//             "'document': [{'e': 'par','c': [{'e': 'text','t': 'Hello reddit, **this should be bold,** the stars around it should not be.',  # noqa'f': [[1, 16, 20]]}]},{'e': 'par','c': [{'e': 'text','t': '> This is text with an arrow in front'}]},{'e': 'blockquote','c': [{'e': 'par','c': [{'e': 'text','t': 'This is a quote',}]}]},{'e': 'par','c': [{'e': 'text','t': 'Here we have something in italics','f': [[2, 0, 33]]}]},{'e': 'par','c': [{'e': 'text','t': '*Here we have something with single-stars around it*'}]},{'e': 'par','c': [{'e': 'text','t': '`Is this a codeblock?`'}]},{'e': 'par','c': [{'e': 'text','t': '~~This should not be strike through~~'}]},{'e': 'par','c': [{'e': 'text','t': 'But this should be','f': [[8, 0, 18]]}]},{'e': 'par','c': [{'e': 'text','t': '[Finally here we have no link](www.example.com)'}]},{'e': 'par','c': [{'e': 'text','t': 'www.thisisalink.com'}]}]"
//         ),
//     );
// }

#[test]
fn lists() {
    rtjson(
        concat!("1.\n   * 1 level [hello](www.reddit.com) nested - ul\n2. 0 levels nested - ol\n3. 0 levels nested - ol\n   1. 1 level nested - ol\n      1. 2 levels nested - ol\n      2. 2 levels nested - ol\n   2. 1 level nested - ol\n      * 2 levels nested - ul\n4. 0 levels nested - ol"),
        concat!(
            "'document': [",
            "{",
            "'e': 'list',",
            "'o': True,",
            "'c': [",
            "{",
            "'e': 'li',",
            "'c': [",
            "{",
            "'e': 'text',",
            "'t': '0 levels nested - ol'",
            "}",
            "]",
            "},",
            "{",
            "'e': 'list',",
            "'o': False,",
            "'c': [",
            "{",
            "'e': 'li',",
            "'c': [",
            "{",
            "'e': 'text',",
            "'t': '1 level nested - ul'",
            "}",
            "]",
            "}",
            "]",
            "},",
            "{",
            "'e': 'li',",
            "'c': [",
            "{",
            "'e': 'text',",
            "'t': '0 levels nested - ol'",
            "}",
            "]",
            "},",
            "{",
            "'e': 'li',",
            "'c': [",
            "{",
            "'e': 'text',",
            "'t': '0 levels nested - ol'",
            "}",
            "]",
            "},",
            "{",
            "'e': 'list',",
            "'o': True,",
            "'c': [",
            "{",
            "'e': 'li',",
            "'c': [",
            "{",
            "'e': 'text',",
            "'t': '1 level nested - ol'",
            "}",
            "],",
            "},",
            "{",
            "'e': 'list',",
            "'o': True,",
            "'c': [",
            "{",
            "'e': 'li',",
            "'c': [",
            "{",
            "'e': 'text',",
            "'t': '2 levels nested - ol'",
            "}",
            "]",
            "},",
            "{",
            "'e': 'li',",
            "'c': [",
            "{",
            "'e': 'text',",
            "'t': '2 levels nested - ol'",
            "}",
            "]",
            "}",
            "]",
            "},",
            "{",
            "'e': 'li',",
            "'c': [",
            "{",
            "'e': 'text',",
            "'t': '1 level nested - ol'",
            "}",
            "]",
            "},",
            "{",
            "'e': 'list',",
            "'o': False,",
            "'c': [",
            "{",
            "'e': 'li',",
            "'c': [",
            "{",
            "'e': 'text',",
            "'t': '2 levels nested - ul'",
            "}",
            "]",
            "}",
            "]",
            "}",
            "]",
            "},",
            "{",
            "'e': 'li',",
            "'c': [",
            "{",
            "'e': 'text',",
            "'t': '0 levels nested - ol'",
            "}",
            "]",
            "}",
            "]",
            "}",
            "]"
        ),
    );

    rtjson(
        concat!("* First item\n* Second item with [a link with **bold** and *italic*](https://reddit.com) and a u/username\n   1. First item\n   2. Second item\n      * First item\n      * Second item"),
        concat!("'document': [",
        "{",
        "'e': 'list',",
        "'o': False,",
        "'c': [",
        "{",
        "'e': 'li',",
        "'c': [",
        "{",
        "'e': 'text',",
        "'t': 'First item'",
        "}",
        "]",
        "},",
        "{",
        "'e': 'li',",
        "'c': [",
        "{",
        "'e': 'text',",
        "'t': 'Second item with '",
        "},",
        "{",
        "'e': 'link',",
        "'u': 'https://reddit.com',",
        "'t': 'a link with bold and italic',",
        "'f': [[1, 12, 4], [2, 21, 6]],",
        "},",
        "{",
        "'e': 'text',",
        "'t': ' and a '",
        "},",
        "{",
        "'e': 'u/',",
        "'t': 'username'",
        "}",
        "]",
        "},",
        "{",
        "'e': 'list',",
        "'o': True,",
        "'c': [",
        "{",
        "'e': 'li',",
        "'c': [",
        "{",
        "'e': 'text',",
        "'t': 'First item'",
        "}",
        "]",
        "},",
        "{",
        "'e': 'li',",
        "'c': [",
        "{",
        "'e': 'text',",
        "'t': 'Second item'",
        "}",
        "]",
        "},",
        "{",
        "'e': 'list',",
        "'o': False,",
        "'c': [",
        "{",
        "'e': 'li',",
        "'c': [",
        "{",
        "'e': 'text',",
        "'t': 'First item'",
        "}",
        "]",
        "},",
        "{",
        "'e': 'li',",
        "'c': [",
        "{",
        "'e': 'text',",
        "'t': 'Second item'",
        "}",
        "]",
        "}",
        "]",
        "}",
        "]",
                "}",
            "]",
        "}",
    "],")
    );
}


#[test]
fn codeblock() {
    rtjson(
        concat!("    function test() {\n      console.log(\"notice the blank line before this function?\");\n    }"),
        concat!(
            "'document':[",
            "{",
            "'e':'code',",
            "'c':[",
            "{",
            "'e':'raw',",
            "'t':'function test() {'",
            "},",
            "{",
            "'e':'raw',",
            "'t':'  console.log(&quot;notice the blank line before this function?&quot;);'",
            "},",
            "{",
            "'e':'raw',",
            "'t':'}'",
            "},",
            "{",
            "'e':'raw',",
            "'t':''",
            "},",
            "],",
            "},",
            "],",
        ),
    );
}
