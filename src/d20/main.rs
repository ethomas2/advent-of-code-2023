use parse::{parse, ModuleType};
use std::collections::HashMap;
use std::collections::VecDeque;
use std::error::Error;
use std::fs;
mod parse;
//
// AllState = Vec<ModuleState>
//
// ModuleState
//      FlipFlop(bool) // on or off
//      Conjunction(Vec<bool>) //TODO: make smallvec
//
// ModuleIdent = &str
//
// Connections:
//      HashMap<ModuleIdent, Vec<ModuleIdent>> // TODO: make smallvec
//
// PulseQueue:
//      Vec<(ModuleIdent, ModuleIdent, bool)
//
// Parse::ModuleType
//      Broadcaster
//      FlipFlop
//      Conjunction
//
// Parse::ModuleMap // map from module ident -> (type for module, list of connections)
//      HashMap<&str, (ParseType, Vec<&str>)>
//
// fn parse -> Connections
//   ...
//
// Part1 Algo:
//      - module_map <- parse
//      - connections <- get connections from module_map
//      - all_state <- initialize **default** all_state from module_map
//      - 1000 times do
//          all_state <- push_button(all_state, connections)
//
//      push_button(state: AllState, connections)
//          queue <- initialize a queue of pulses
//          push button (enqueue all the initial pulses)
//
//          while !queue.is_empty()
//              pulse <- queue.pop()
//              process pulse
//                  - match on pulse type and
//                  - update ModuleState
//                  - get new pulses
//              put new pulses on the queue
//              yield pulses // TODO time to use generators for the first time??? If not just append to a vec

type ModuleIdent<'a> = &'a str;
type Connections<'a, 'b> = HashMap<ModuleIdent<'a>, &'b Vec<ModuleIdent<'a>>>;

#[derive(Debug)]
enum ModuleStateVal<'a> {
    Broadcaster(Vec<ModuleIdent<'a>>),
    FlipFlop(bool),
    Conjunction(Vec<(ModuleIdent<'a>, bool)>),
}

type AllState<'a> = HashMap<ModuleIdent<'a>, ModuleStateVal<'a>>;

#[derive(Debug)]
struct Pulse<'a> {
    src: ModuleIdent<'a>,
    dst: ModuleIdent<'a>,
    high_or_low: bool,
}

fn reverse_map<T1, T2>(map: &HashMap<T1, &Vec<T2>>) -> HashMap<T2, Vec<T1>>
where
    T1: Eq + Clone + std::hash::Hash,
    T2: Eq + Clone + std::hash::Hash,
{
    let mut h = HashMap::new();
    for (x, ys) in map {
        for y in ys.iter() {
            h.entry(y.clone()).or_insert(vec![]).push(x.clone());
        }
    }
    h
}

// TODO: make this an iterator
fn update_state<'a, 'b>(
    state: &'b mut AllState<'a>,
    connections: &'b Connections<'a, 'b>,
    pulse: &Pulse<'a>,
) -> Vec<Pulse<'a>> {
    let Pulse {
        src,
        dst,
        high_or_low,
    } = pulse;
    let emptyvec = vec![];
    let downstream_modules = *connections.get(dst).unwrap_or(&&emptyvec);

    let module_state = match state.get_mut(dst) {
        None => {
            // eprintln!("Warning: got pulse to blackhole dst {}", dst);
            return vec![];
        }
        Some(x) => x,
    };
    let new_pulses: Vec<_> = match module_state {
        ModuleStateVal::Broadcaster(connections) => {
            debug_assert!(dst == &"broadcaster");
            connections
                .iter()
                .map(|downstream| Pulse {
                    src: dst,
                    dst: downstream,
                    high_or_low: *high_or_low,
                })
                .collect()
        }
        ModuleStateVal::FlipFlop(ref mut flip_flop_state) => match high_or_low {
            true => vec![],
            false => {
                *flip_flop_state = !*flip_flop_state;
                downstream_modules
                    .iter()
                    .map(|ds_mod| Pulse {
                        src: dst,
                        dst: ds_mod,
                        high_or_low: *flip_flop_state,
                    })
                    .collect()
            }
        },

        ModuleStateVal::Conjunction(ref mut recent_pulses) => {
            let (_, val) = recent_pulses
                .iter_mut()
                .find(|(ident, _)| ident == src)
                .unwrap();
            *val = *high_or_low;

            let new_pulses_val = !recent_pulses.iter().all(|(_, val)| *val);
            downstream_modules
                .iter()
                .map(|module| Pulse {
                    src: dst,
                    dst: module,
                    high_or_low: new_pulses_val,
                })
                .collect()
        }
    };
    new_pulses
}

fn push_button<'a, 'b>(
    state: &'b mut AllState<'a>,
    connections: &'b Connections<'a, 'b>,
) -> impl Iterator<Item = Pulse<'a>> + 'b
where
    'a: 'b,
{
    // TODO: might have to send a button pulse
    let mut queue = VecDeque::new();

    // init the initial pulses
    // let broadcaster_connetions = connections.get("broadcaster").unwrap();
    // queue.extend(broadcaster_connetions.iter().map(|&dst| Pulse {
    //     src: "broadcaster",
    //     dst,
    //     high_or_low: false,
    // }));

    queue.push_front(Pulse {
        src: "button",
        dst: "broadcaster",
        high_or_low: false,
    });

    std::iter::from_fn(move || {
        let pulse = queue.pop_front()?;
        let new_pulses = update_state(state, &connections, &pulse);
        queue.extend(new_pulses.into_iter());
        Some(pulse)
    })
}

fn main() -> Result<(), Box<dyn Error>> {
    let content = fs::read_to_string("src/d20/input")?;
    let (_, mmap) = parse(&content).unwrap();
    println!("Parsed val {:#?}", mmap);
    let connections: Connections<'_, '_> = mmap.iter().map(|(src, (_, dst))| (*src, dst)).collect();
    let rev_connections = reverse_map(&connections);
    let mut state: AllState<'_> = mmap
        .iter()
        .map(|(src, (module_type, _))| {
            let key = src;
            let val = match *module_type {
                ModuleType::Broadcaster => {
                    ModuleStateVal::Broadcaster((*connections.get("broadcaster").unwrap()).clone())
                }
                ModuleType::FlipFlop => ModuleStateVal::FlipFlop(false),
                ModuleType::Conjunction => ModuleStateVal::Conjunction(
                    rev_connections
                        .get(src)
                        .unwrap()
                        .iter()
                        .map(|&prev| (prev, false))
                        .collect(),
                ),
            };
            (*key, val)
        })
        .collect();

    // TODO: Investigate why I have to do this collect(). Something about can't allow captured
    // variables to escpae a closure
    let (mut low_pulses, mut high_pulses): (usize, usize) = (0, 0);
    for pulse in (0..1000).flat_map(|_| push_button(&mut state, &connections).collect::<Vec<_>>()) {
        if pulse.high_or_low {
            high_pulses += 1;
        } else {
            low_pulses += 1;
        }
    }
    println!(
        "high_pulses {} low_pulses::{} product::{}",
        high_pulses,
        low_pulses,
        high_pulses * low_pulses
    );

    Ok(())
}
