mod parse;
use either::Either;
use parse::parse;
use std::error::Error;
use std::fs;

#[derive(Debug, Clone, PartialEq, Eq)]
enum AcceptReject {
    Accept,
    Reject,
}

#[derive(Debug, Clone)]
enum PartAttr {
    X,
    M,
    A,
    S,
}

#[derive(Debug, Clone)]
enum GTorLT {
    GT,
    LT,
}

#[derive(Debug, Clone)]
struct WorkflowRule<'a> {
    attr: PartAttr,
    gtlt: GTorLT,
    val: usize,
    action: Either<AcceptReject, &'a str>,
}

impl<'a> WorkflowRule<'a> {
    fn applies(&self, part: &Part) -> bool {
        let val_of_attr = match self.attr {
            PartAttr::X => part.x,
            PartAttr::M => part.m,
            PartAttr::A => part.a,
            PartAttr::S => part.s,
        };
        let matches = match self.gtlt {
            GTorLT::GT => val_of_attr > self.val,
            GTorLT::LT => val_of_attr < self.val,
        };
        if matches {
            return true;
        }
        return false;
    }
}

#[derive(Debug, Clone)]
struct Workflow<'a> {
    name: &'a str,
    rules: Vec<WorkflowRule<'a>>,
    default: Either<AcceptReject, &'a str>,
}

impl<'a> Workflow<'a> {
    fn apply(&self, part: &Part) -> Either<AcceptReject, &'a str> {
        for rule in &self.rules {
            if rule.applies(part) {
                return rule.action.clone();
            }
        }
        return self.default.clone();
    }
}

#[derive(Debug, Clone)]
struct Part {
    x: usize,
    m: usize,
    a: usize,
    s: usize,
}

fn send_part(workflows: &Vec<Workflow>, part: &Part) -> AcceptReject {
    let mut wf_ident = Either::Right("in");
    loop {
        match wf_ident {
            Either::Right(name) => {
                let wf = workflows
                    .iter()
                    .find(|Workflow { name: wf_name, .. }| *wf_name == name);
                let wf = wf.unwrap(); // TODO: reutrn Err or something
                wf_ident = wf.apply(part);
            }
            Either::Left(acc_rej) => return acc_rej,
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // types
    //      Workflow { name, rules: Vec<WorkflowRule>, default: (A/R/Send) }
    //      WorkflowRule { partattr, GTorLT, usize , (A/R/Send) }
    //      Part { x, m, a, s }
    //
    // parse input
    //      parse_workflows -> Vec<Workflow>
    //      parse_parts -> Vec<Part>
    //      parse_all -> (Vec<Workflow>, Vec<Part>)
    //
    // send_part(part) -> Accept/Reject
    //      wf_ident = "in"
    //      while wf_ident not in [A, R]
    //          wf_ident = get_workflow_rule(state)
    //          wf_ident = wf_rule(part)
    //      return wf_ident
    //
    //  parts.filter(|part| send_part(part) in [A, R])
    //  add all the numbers from the accepted parts

    let content = fs::read_to_string("src/d19/input")?;

    let (workflows, parts) = parse(&content)?;
    let total: usize = parts
        .iter()
        .filter(|part| send_part(&workflows, part) == AcceptReject::Accept)
        .map(|part| {
            let Part { x, m, a, s } = part.clone();
            x + m + a + s
        })
        .sum();
    println!("p1 {}", total);
    Ok(())
}
