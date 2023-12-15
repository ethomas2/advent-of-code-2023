use std::error::Error;
use std::fs;
use std::str::FromStr;

fn hash(input: &str) -> u8 {
    let mut value: u8 = 0;
    for ch in input.chars() {
        value = 17u8.wrapping_mul(value.wrapping_add(ch as u8));
    }
    value
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Instruction<'a> {
    Set { label: &'a str, val: u8 },
    Del { label: &'a str },
}

impl<'a> Instruction<'a> {
    fn parse(content: &'a str) -> Result<Instruction<'a>, String> {
        if content.contains("=") {
            let mut x = content.split("=");
            let label = x.next().ok_or("oh no".to_string())?;
            let val: u8 = x
                .next()
                .ok_or("oh no".to_string())?
                .parse()
                .map_err(|_| "oh no".to_string())?;
            if x.next() != None {
                return Err("oh no".to_string());
            }
            return Ok(Instruction::Set { label, val });
        } else if content.ends_with("-") {
            return Ok(Instruction::Del {
                label: &content[0..(content.len() - 1)],
            });
        } else {
        }
        todo!()
    }
}

type HashMapState<'a> = [Vec<(&'a str, u8)>; 256];

fn run_hashmap<'a>(instructions: &'a [Instruction]) -> HashMapState<'a> {
    let mut state: HashMapState = std::array::from_fn(|_| Vec::new());
    for instruction in instructions {
        match instruction {
            &Instruction::Set { label, val } => {
                let _box = &mut state[hash(label) as usize];
                let item = _box
                    .iter()
                    .enumerate()
                    .find_map(|(idx, &(lbl, _))| (lbl == label).then_some(idx));
                match item {
                    Some(idx) => {
                        _box[idx] = (label, val);
                    }
                    None => state[hash(label) as usize].push((label, val)),
                }
            }
            Instruction::Del { label } => {
                let _box = &mut state[hash(label) as usize];
                let item = _box
                    .iter()
                    .enumerate()
                    .find_map(|(idx, (lbl, _))| (lbl == label).then_some(idx));
                if let Some(idx) = item {
                    _box.remove(idx);
                }
            }
        }
    }
    state
}

fn total_focusing_power(state: &HashMapState) -> usize {
    let x: usize = state
        .iter()
        .enumerate()
        .flat_map(|(box_idx, vec)| {
            vec.iter()
                .enumerate()
                .map(move |(slot_idx, &(_, val))| (box_idx, slot_idx, val))
        })
        .map(|(box_idx, slot_idx, val)| (1 + box_idx) * (1 + slot_idx) * (val as usize))
        .sum();

    x
}

fn main() -> Result<(), Box<dyn Error>> {
    // part 1
    {
        let content = fs::read_to_string("src/d15/input")?;
        let inputs = content.trim().split(",");
        let s1: usize = inputs.map(|x| hash(x) as usize).sum();
        println!("s1 {}", s1);
    }

    // part 2
    {
        let content = fs::read_to_string("src/d15/input")?;
        let inputs = content.trim().split(",");
        let instructions: Vec<Instruction> =
            inputs.map(Instruction::parse).collect::<Result<_, _>>()?;
        let state = run_hashmap(&instructions);
        println!("{:?}", state);
        println!("s2 {}", total_focusing_power(&state));
    }
    Ok(())
}
