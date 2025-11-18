use std::{env};

const N: usize = 624;
const M: usize = 397;
const MATRIX_A: u32 = 0x9908b0df;
const UPPER_MASK: u32 = 0x80000000;
const LOWER_MASK: u32 = 0x7fffffff;
const MAX_GRID_VALUE: u32 = 50;

struct MT {
    mt: [u32; N],
    index: usize,
}

impl MT {
    fn new(seed: u32) -> Self {
        let mut mt = [0u32; N];
        mt[0] = seed;
        for i in 1..N {
            let prev = mt[i - 1];
            mt[i] = 1812433253u32
                .wrapping_mul(prev ^ (prev >> 30))
                .wrapping_add(i as u32);
        }
        MT { mt, index: N }
    }

    fn twist(&mut self) {
        for i in 0..N {
            let x = (self.mt[i] & UPPER_MASK) | (self.mt[(i + 1) % N] & LOWER_MASK);
            let mut x_a = x >> 1;
            if x % 2 != 0 {
                x_a ^= MATRIX_A;
            }
            self.mt[i] = self.mt[(i + M) % N] ^ x_a;
        }
        self.index = 0;
    }

    fn gen_u32(&mut self) -> u32 {
        if self.index >= N {
            self.twist();
        }
        let mut y = self.mt[self.index];
        self.index += 1;

        // Tempering
        y ^= y >> 11;
        y ^= (y << 7) & 0x9d2c5680;
        y ^= (y << 15) & 0xefc60000;
        y ^= y >> 18;
        y
    }
}

pub struct Buffer {
    now_state: Vec<u32>,
    first_state: Vec<u32>,
    b_state: Vec<u32>,
    operations: Vec<u32>,
    applications: Vec<u32>,
    n: usize,
    now_time: usize,
    revisions: Vec<usize>,
    changes: Vec<usize>,
    change_idx: Vec<usize>,
    change_ptr: usize,
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer {
            now_state: vec![0; 3600],
            first_state: vec![0; 3600],
            b_state: vec![0; 3600],
            operations: vec![0; 12000],
            now_time: 0,
            n: 0,
            applications: vec![0; 0],
            revisions: vec![0; 0],
            changes: vec![0; 0],
            change_idx: vec![0; 0],
            change_ptr: 0,
        }
    }

    pub fn random_gen(&mut self, seed: u32) {
        let mut mt = MT::new(seed);
        let n = (mt.gen_u32() % 46 + 15) as usize;
        self.n = n;
        let n32 = n as u32;
        for i in 0..n {
            for j in 0..n {
                let val = mt.gen_u32() % MAX_GRID_VALUE;
                self.now_state[i * n + j] = val;
                self.first_state[i * n + j] = val;
            }
        }
        for i in 0..n {
            for j in 0..n {
                let val = mt.gen_u32() % MAX_GRID_VALUE;
                self.b_state[i * n + j] = val;
            }
        }
        for i in 0..3000 {
            let op1 = mt.gen_u32() % n32;
            let op2 = mt.gen_u32() % n32;
            let op3 = mt.gen_u32() % MAX_GRID_VALUE;
            let op4 = mt.gen_u32() % MAX_GRID_VALUE;
            self.operations[i * 4] = op1 + 1;
            self.operations[i * 4 + 1] = op2 + 1;
            self.operations[i * 4 + 2] = op3;
            self.operations[i * 4 + 3] = op4;
        }
        self.now_time = 0;
        self.applications.clear();
        self.revisions.clear();
        self.changes.clear();
        self.change_idx.clear();
        self.change_ptr = 0;
    }

    pub fn set_first_state(&mut self, state: Vec<u32>, n: usize) {
        self.n = n;
        self.first_state = state.clone();
        self.now_state = state;
        self.now_time = 0;
        self.applications.clear();
        self.revisions.clear();
        self.changes.clear();
        self.change_idx.clear();
        self.change_ptr = 0;
    }

    pub fn set_b_state(&mut self, state: Vec<u32>) {
        self.b_state = state;
        self.now_time = 0;
        self.applications.clear();
        self.revisions.clear();
        self.changes.clear();
        self.change_idx.clear();
        self.change_ptr = 0;
    }

    pub fn set_op(&mut self, op: Vec<u32>) {
        self.operations = op;
        self.now_time = 0;
        self.applications.clear();
        self.revisions.clear();
        self.changes.clear();
        self.change_idx.clear();
        self.change_ptr = 0;
    }

    pub fn to_string(&self) -> String {
        let n = self.n;
        let t = self.operations.len() / 4;
        let mut data = String::new();
        data.push_str(&format!("{} {}\n", n, t));
        for i in 0..n {
            for j in 0..n {
                data.push_str(&format!("{} ", self.first_state[i * n + j]));
            }
            data.push('\n');
        }
        for i in 0..n {
            for j in 0..n {
                data.push_str(&format!("{} ", self.b_state[i * n + j]));
            }
            data.push('\n');
        }
        for op_chunk in self.operations.chunks(4) {
            data.push_str(&format!(
                "{} {} {} {}\n",
                op_chunk[0], op_chunk[1], op_chunk[2], op_chunk[3]
            ));
        }
        data
    }

    pub fn update_applications(&mut self, applications: Vec<u32>) {
        self.applications = applications;
        self.now_time = 0;
        self.now_state = self.first_state.clone();
        self.revisions = vec![0; self.applications.len() / 3 + 1];
        self.changes.clear();
        self.change_idx.clear();
        for (time, chunk) in self.applications.chunks(3).enumerate() {
            let op_id = (chunk[0] as usize) * 4;
            let x = chunk[1];
            let y = chunk[2];
            let h = self.operations[op_id];
            let w = self.operations[op_id + 1];
            let a = self.operations[op_id + 2];
            let b = self.operations[op_id + 3];
            for i in 0..h {
                for j in 0..w {
                    let idx = ((x + i) as usize) * self.n + (y + j) as usize;
                    if idx >= self.n * self.n {
                        unreachable!();
                    }
                    if self.now_state[idx] == a {
                        self.now_state[idx] = b;
                        self.changes.push(idx);
                        self.change_idx.push(op_id);
                        self.revisions[time + 1] = self.changes.len();
                    }
                }
            }
        }
        self.change_ptr = self.changes.len();
        self.now_time = self.applications.len() / 3;
    }

    pub fn now_state_ptr(&self) -> *const u32 {
        self.now_state.as_ptr()
    }

    pub fn first_state_ptr(&self) -> *const u32 {
        self.first_state.as_ptr()
    }

    pub fn b_state_ptr(&self) -> *const u32 {
        self.b_state.as_ptr()
    }

    pub fn set_time(&mut self, time: usize) -> bool {
        if time > self.applications.len() / 3 + 1 {
            return false;
        }
        while self.change_ptr < self.revisions[time] {
            let idx = self.changes[self.change_ptr];
            let op_id = self.change_idx[self.change_ptr];
            let a = self.operations[op_id + 2];
            let b = self.operations[op_id + 3];
            assert_eq!(self.now_state[idx], a);
            self.now_state[idx] = b;
            self.change_ptr += 1;
        }
        while self.change_ptr > self.revisions[time] {
            self.change_ptr -= 1;
            let idx = self.changes[self.change_ptr];
            let op_id = self.change_idx[self.change_ptr];
            let a = self.operations[op_id + 2];
            let b = self.operations[op_id + 3];
            assert_eq!(self.now_state[idx], b);
            self.now_state[idx] = a;
        }
        self.now_time = time;
        true
    }

    pub fn get_time(&self) -> usize {
        self.now_time
    }
}

fn usage(args: &Vec<String>) {
    eprintln!("Usage: {} gen once <output_dir> <seed>", args[0]);
    eprintln!("Usage: {} gen list <output_dir> <seed_list_file>", args[0]);
    eprintln!("Usage: {} gen random <output_dir> <seed_count>", args[0]);
    eprintln!("Usage: {} test dir <input_dir> <command>\n", args[0]);
    eprintln!("Example1: {} gen list ./inputs seed.txt", args[0]);
    eprintln!("Example1: {} test dir ./inputs ./Main", args[0]);
}

fn main() {
    let args = env::args().collect::<Vec<String>>();
    if args.len() < 2 {
        usage(&args);
        return;
    }
    let command = &args[1];
    if command == "gen" {
        let sub_command = &args[2];
        if sub_command == "once" {
            let output_dir = &args[3];
            let seed: u32 = args[4].parse().unwrap();
            let mut buffer = Buffer::new();
            buffer.random_gen(seed);
            let output_path = format!("{}/input_{}.txt", output_dir, seed);
            std::fs::write(output_path, buffer.to_string()).unwrap();
        } else if sub_command == "list" {
            let output_dir = &args[3];
            let seed_list_file = &args[4];
            let seed_list = std::fs::read_to_string(seed_list_file).unwrap();
            for line in seed_list.lines() {
                let seed: u32 = line.trim().parse().unwrap();
                let mut buffer = Buffer::new();
                buffer.random_gen(seed);
                let output_path = format!("{}/input_{}.txt", output_dir, seed);
                std::fs::write(output_path, buffer.to_string()).unwrap();
            }
        } else if sub_command == "random" {
            let output_dir = &args[3];
            let seed_count: u32 = args[4].parse().unwrap();
            for _ in 0..seed_count {
                let seed = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos() as u32;
                let mut buffer = Buffer::new();
                buffer.random_gen(seed);
                let output_path = format!("{}/input_{}.txt", output_dir, seed);
                std::fs::write(output_path, buffer.to_string()).unwrap();
            }
        } else {
            usage(&args);  
        }
    } else if command == "test" {
        let sub_command = &args[2];
        if sub_command == "dir" {
            let mut sum = 0;
            let mut count = 0;
            let input_dir = &args[3];
            let test_command = &args[4];
            let entries = std::fs::read_dir(input_dir).unwrap();
            for entry in entries {
                let entry = entry.unwrap();
                let path = entry.path();
                if path.is_file() {
                    let file_sys = std::fs::File::open(&path).unwrap();
                    let output = std::process::Command::new(test_command).stdin(std::process::Stdio::from(file_sys)).output().unwrap();

                    let input = std::fs::read_to_string(&path).unwrap();
                    let mut lines = input.lines();
                    let first_line = lines.next().unwrap().split_whitespace();
                    
                    let n: usize = first_line.clone().nth(0).unwrap().parse().unwrap();
                    let t: usize = first_line.clone().nth(1).unwrap().parse().unwrap();
                    let mut a_state = vec![0u32; n * n];
                    let mut b_state = vec![0u32; n * n];
                    let mut operations = vec![0u32; t * 4];

                    for i in 0..n {
                        let line = lines.next().unwrap();
                        for (j, val) in line.split_whitespace().enumerate() {
                            a_state[i * n + j] = val.parse().unwrap();
                        }
                    }
                    for i in 0..n {
                        let line = lines.next().unwrap();
                        for (j, val) in line.split_whitespace().enumerate() {
                            b_state[i * n + j] = val.parse().unwrap();
                        }
                    }
                    for i in 0..t {
                        let line = lines.next().unwrap();
                        let mut iter = line.split_whitespace();
                        operations[i * 4] = iter.next().unwrap().parse().unwrap();
                        operations[i * 4 + 1] = iter.next().unwrap().parse().unwrap();
                        operations[i * 4 + 2] = iter.next().unwrap().parse().unwrap();
                        operations[i * 4 + 3] = iter.next().unwrap().parse().unwrap();
                    }
                    let mut buff = Buffer::new();
                    buff.set_first_state(a_state, n);
                    buff.set_b_state(b_state);
                    buff.set_op(operations);

                    let output_str = String::from_utf8_lossy(&output.stdout);
                    let mut output_lines = output_str.lines();
                    let line_len = output_lines.next().unwrap().parse::<usize>().unwrap();
                    let applications: Vec<u32> = output_str.lines().map(|line| {
                        line.split_whitespace().map(|val: &str| val.parse::<u32>().unwrap()).collect::<Vec<u32>>()
                    }).flatten().collect::<Vec<u32>>().into_iter().skip(1).collect();
                    assert_eq!(applications.len(), line_len * 3);
                    buff.update_applications(applications);

                    let mut score  = (n * n + buff.get_time()) as u32;

                    for i in 0..n {
                        for j in 0..n {
                            score += ((buff.now_state[i * n + j] as i32) - (buff.b_state[i * n + j] as i32)).pow(2) as u32;
                        }
                    }

                    let last_score = (1e9 * ((n * n) as f64) / (score as f64)).round();
                    println!("Score: {} (Error: {} in {:?})", last_score, score, path);
                    sum += last_score as u32;
                    count += 1;
                }
            }
            println!("Total Score: {}", sum);
            println!("Average Score: {}", sum as f64 / count as f64);
        }
    }
}
