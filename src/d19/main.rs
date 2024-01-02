mod ndrange;
mod parse;
use crate::ndrange::{NDRange, Range};
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

type WorkflowIdentifier<'a> = Either<AcceptReject, &'a str>;

#[derive(Debug, Clone)]
struct WorkflowRule<'a> {
    attr: PartAttr,
    gtlt: GTorLT,
    val: usize,
    dst: WorkflowIdentifier<'a>,
}

#[derive(Debug, Clone)]
struct Workflow<'a> {
    name: &'a str,
    rules: Vec<WorkflowRule<'a>>,
    default: WorkflowIdentifier<'a>,
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

impl<'a> Workflow<'a> {
    fn apply(&self, part: &Part) -> WorkflowIdentifier<'a> {
        for rule in &self.rules {
            if rule.applies(part) {
                return rule.dst.clone();
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

/// Compute the answer for part 1
fn part1(workflows: &Vec<Workflow<'_>>, parts: &Vec<Part>) -> usize {
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
    let total: usize = parts
        .iter()
        .filter(|part| send_part(&workflows, part) == AcceptReject::Accept)
        .map(|part| {
            let Part { x, m, a, s } = part.clone();
            x + m + a + s
        })
        .sum();
    total
}

fn part2(workflows: &Vec<Workflow<'_>>, lbound: usize, ubound: usize) -> usize {
    let start = NDRange::new([
        Range::new(lbound, ubound),
        Range::new(lbound, ubound),
        Range::new(lbound, ubound),
        Range::new(lbound, ubound),
    ]);
    split_range_through_graph(start, Either::Right("in"), workflows)
        .into_iter()
        .filter(|(_, acc_rej)| *acc_rej == AcceptReject::Accept)
        .map(|(ndrange, _)| {
            ndrange
                .0
                .iter()
                .map(|range| range.end - range.start)
                .product::<usize>()
        })
        .sum()
}

// TODO: could you get this to return an iterator?
/// Input: A range and a part of the graph to "DISPERSE" taht range through.
/// Output: The set of all the tiny ranges
fn split_range_through_graph(
    range: NDRange,
    wf_ident: WorkflowIdentifier,
    workflows: &Vec<Workflow<'_>>,
) -> Vec<(NDRange, AcceptReject)> {
    // Let's call this function DISPERSE. Psuedocode:
    //
    // DISPERSE(wf_ident, range) -> Vec<(Range, AcceptReject)> {
    //      workflow <- get the workflow for this ident
    //      result <- empty array
    //      for rule in wf.rules
    //          - pluck off the part of this NDRange that this rule applies to
    //          - recursively send that through the graph (send it through the workflow for this
    //          rule)
    //          - set "remain" to be whatever's left after plucking off this part
    //
    //      - send "remain" through the default for this workflow
    if matches!(wf_ident, Either::Left(_)) {
        return vec![(range, wf_ident.unwrap_left())];
    }
    if range.is_empty() {
        return Vec::new();
    }
    let wf_ident = wf_ident.unwrap_right();
    let workflow = workflows
        .iter()
        .find(|Workflow { name: wf_name, .. }| *wf_name == wf_ident)
        .expect(&format!("couldn't find workflow with name {}", wf_ident)); // TODO: no expect

    let result = {
        let mut remain = range;
        let mut result: Vec<_> = Vec::new();

        // for each rule, split off the piece of range handled by this rule and send it through
        // it's part of the workflow graph
        for rule in &workflow.rules {
            let ((r1, r1_dst), r2) = split(remain, rule);
            result.extend(split_range_through_graph(r1, r1_dst, workflows));
            remain = r2;
        }

        // whatever is remaining goes to default
        result.extend(split_range_through_graph(
            remain,
            workflow.default.clone(),
            workflows,
        ));

        // TODO: this might not be necessary?
        result = result
            .into_iter()
            .filter(|(range, _)| !range.is_empty())
            .collect();
        result
    };

    result
}

/// Given a range and a rule, split the range into (this_range, remain) where
///     this_range :: is part of the range handled by this rule (possibly empty)
///     remain :: is the remaining part not handled by this rule (also possibly empty)
///
/// Since this_range is all handled by one rule, also give the destination for this_range
fn split<'a, 'b>(
    range: NDRange,
    rule: &'b WorkflowRule<'a>,
) -> ((NDRange, WorkflowIdentifier<'a>), NDRange) {
    let dimension: usize = match rule.attr {
        PartAttr::X => 0,
        PartAttr::M => 1,
        PartAttr::A => 2,
        PartAttr::S => 3,
    };

    // Suppose this dimension's range is [0, 10)
    //      if the rule is < 5 then this_range is the left part of
    //              **[0, 5)**, [5, 10)
    //      if the rule is > 5 then this_range is the right part of
    //              [0, 6), **[6, 10)**
    let (this_range, remain) = match rule.gtlt {
        GTorLT::LT => range.split(dimension, rule.val),
        GTorLT::GT => {
            let (remain, this_range) = range.split(dimension, rule.val + 1);
            (this_range, remain)
        }
    };

    ((this_range, rule.dst.clone()), remain)
}

fn main() -> Result<(), Box<dyn Error>> {
    let content = fs::read_to_string("src/d19/input")?;
    let (workflows, parts) = parse(&content)?;

    println!("p1 {}", part1(&workflows, &parts));
    println!("p2 {}", part2(&workflows, 1, 4001));
    Ok(())
}
