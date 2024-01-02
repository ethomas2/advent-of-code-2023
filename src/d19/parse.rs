use super::*;

use either::Either;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete,
    character::complete::{alpha1, newline},
    combinator::all_consuming,
    error::{ErrorKind, ParseError},
    multi::{many0, separated_list1},
    sequence::tuple,
    IResult,
};

#[derive(Debug, PartialEq)]
pub struct CustomError(String);

impl<I> ParseError<I> for CustomError {
    fn from_error_kind(_: I, kind: ErrorKind) -> Self {
        CustomError(format!("{:?}", kind))
    }

    fn append(_: I, _: ErrorKind, other: Self) -> Self {
        other
    }
}

impl From<nom::Err<nom::error::Error<&str>>> for CustomError {
    fn from(error: nom::Err<nom::error::Error<&str>>) -> Self {
        CustomError(format!("{}", error))
    }
}

type ParseResult<'a, T> = IResult<&'a str, T, CustomError>;

/// Parse a rule from a workflow. E.g. a<2006:qkq or m>2090:A from px{a<2006:qkq,m>2090:A,rfg}
pub fn parse_rule<'a>(input: &'a str) -> ParseResult<WorkflowRule<'a>> {
    let (input, attr) = alpha1(input)?;
    let (input, gtlt) = alt((tag("<"), tag(">")))(input)?;
    let (input, val) = complete::u64(input)?;
    let (input, _) = tag(":")(input)?;
    let (input, dst) = alpha1(input)?;
    let attr = match attr {
        "x" => PartAttr::X,
        "m" => PartAttr::M,
        "a" => PartAttr::A,
        "s" => PartAttr::S,
        _ => Err(nom::Err::Error(CustomError("oh no".to_owned())))?,
    };
    let gtlt = match gtlt {
        ">" => GTorLT::GT,
        "<" => GTorLT::LT,
        _ => Err(nom::Err::Error(CustomError("oh no".to_owned())))?,
    };
    let dst = match dst {
        "A" => Either::Left(AcceptReject::Accept),
        "R" => Either::Left(AcceptReject::Reject),
        _ => Either::Right(dst),
    };
    let val: usize = val.try_into().unwrap();
    Ok((
        input,
        WorkflowRule {
            attr,
            gtlt,
            val,
            dst,
        },
    ))
}

/// Parse a workflow. E.g. px{a<2006:qkq,m>2090:A,rfg}
pub fn parse_workflow<'a>(input: &'a str) -> ParseResult<Workflow<'a>> {
    let (input, name) = alpha1(input)?;
    let (input, _) = tag("{")(input)?;

    let (input, rules) = separated_list1(tag(","), parse_rule)(input)?;
    let (input, _) = tag(",")(input)?;
    let (input, default) = alpha1(input)?;
    let (input, _) = tag("}")(input)?;
    let default = match default {
        "A" => Either::Left(AcceptReject::Accept),
        "R" => Either::Left(AcceptReject::Reject),
        _ => Either::Right(default),
    };
    Ok((
        input,
        Workflow {
            name,
            rules,
            default,
        },
    ))
}

/// Parse a part. E.g. {x=787,m=2655,a=1222,s=2876}
pub fn parse_part<'a>(input: &'a str) -> ParseResult<Part> {
    let (input, _) = tag("{")(input)?;
    // TODO: would be nice if you didn't allocate a list
    let (input, attrs_list) =
        separated_list1(tag(","), |input: &'a str| -> ParseResult<(&str, usize)> {
            let (input, (name, _, val)) = tuple((
                alt((tag("x"), tag("m"), tag("a"), tag("s"))),
                tag("="),
                complete::u64,
            ))(input)?;
            Ok((input, (name, val as usize)))
        })(input)?;

    if attrs_list.len() != 4 {
        return Err(nom::Err::Error(CustomError("too long".to_owned())));
    }

    let (input, _) = tag("}")(input)?;

    let (x, m, a, s): (usize, usize, usize, usize) = {
        let (mut x, mut m, mut a, mut s) = (None, None, None, None);
        for (ident, val) in attrs_list {
            match ident {
                "x" => x = Some(val),
                "m" => m = Some(val),
                "a" => a = Some(val),
                "s" => s = Some(val),
                _ => return Err(nom::Err::Error(CustomError("unknown char".to_owned()))),
            }
        }
        (
            x.ok_or_else(|| nom::Err::Error(CustomError("Missing attribute: x".to_owned())))?,
            m.ok_or_else(|| nom::Err::Error(CustomError("Missing attribute: m".to_owned())))?,
            a.ok_or_else(|| nom::Err::Error(CustomError("Missing attribute: a".to_owned())))?,
            s.ok_or_else(|| nom::Err::Error(CustomError("Missing attribute: s".to_owned())))?,
        )
    };
    Ok((input, Part { x, m, a, s }))
}

pub fn parse<'a>(input: &'a str) -> Result<(Vec<Workflow<'a>>, Vec<Part>), nom::Err<CustomError>> {
    let (_, result) = all_consuming(|input| {
        let (input, workflows) = separated_list1(newline, parse_workflow)(input)?;
        let (input, _) = many0(newline)(input)?;
        let (input, parts) = separated_list1(newline, parse_part)(input)?;
        let (input, _) = many0(newline)(input)?;
        Ok((input, (workflows, parts)))
    })(input)?;
    Ok(result)
}
