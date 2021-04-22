use nom::{
    bytes::complete::{escaped, is_not, tag, take_while},
    character::complete::{alphanumeric1, char, none_of, one_of},
    combinator::{cut, map, opt, value},
    error::{context, convert_error, ContextError, ErrorKind, ParseError, VerboseError},
    multi::separated_list0,
    regexp::str::re_match,
    sequence::{delimited, preceded, separated_pair, terminated},
    Err, IResult,
};

pub type BoxError = std::boxed::Box<dyn std::error::Error + std::marker::Send + std::marker::Sync>;

pub struct Command<'a> {
    command: &'a str,
    args: Vec<&'a str>,
}

pub enum Argument<'a> {
    Text(&'a str),
    Env(&'a str),
}

/*
fn string<'a>(i: &'a str) -> IResult<&'a str, &'a str> {
    context(
        "string",
        preceded(char('\"'), cut(terminated(parse_str, char('\"')))),
    )(i)
}

fn incomplete_string<'a>(i: &'a str) -> IResult<&'a str, &'a str> {
    preceded(char('\"'), parse_str)(i)
}*/

fn sp<'a>(i: &'a str) -> IResult<&'a str, &'a str> {
    let chars = " \t";
    take_while(move |c| chars.contains(c))(i)
}

fn arg<'a>(i: &'a str) -> IResult<&'a str, &'a str> {
    escaped(none_of("#<>$`&*\'\"|?= \n\t\r"), '\\', one_of("\"n\\"))(i)
}

fn env_token<'a>(i: &'a str) -> IResult<&'a str, &'a str> {
    let re = regex::Regex::new(r"\$\w+").unwrap();
    re_match(re)(i)
}

fn env_variable<'a>(i: &'a str) -> IResult<&'a str, &'a str> {
    context("env_variable", preceded(sp, cut(terminated(env_token, sp))))(i)
}

fn parse_cmd<'a>(i: &'a str) -> IResult<&'a str, &'a str> {
    context("cmd", preceded(sp, cut(terminated(arg, sp))))(i)
}

fn parse_args<'a>(i: &'a str) -> IResult<&'a str, Vec<&'a str>> {
    context(
        "args",
        preceded(
            sp,
            cut(terminated(
                separated_list0(none_of(""), preceded(sp, cut(terminated(arg, sp)))),
                sp,
            )),
        ),
    )(i)
}

//fn parse_command<'a>(i: &'a str) -> IResult<&'a str, &'a str> {
//}

fn main() {
    /*println!("{:?}", incomplete_string("\"hello poggers dude"));
    println!("{:?}", string("\"hell    o\""));
    println!("{:?}", parse_str("hello world"));*/
    
    println!("{:?}", arg("hello|world"));
    println!("{:?}", sp(""));
    println!("{:?}", parse_args(""))
}
