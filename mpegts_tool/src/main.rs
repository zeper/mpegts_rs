use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::env;


struct ts_packet {
    transport_error_indicator: bool,
    payload_unit_start_indicator: bool,
    transport_priority: bool,
    pid: u16,
    transport_scrambling_control: u8,
    adaptation_field_control: u8,
    continuity_counter: u8,
    data:[u8;184],
}


struct pat {
    version: u8,
    programs: Vec<program>,
}

struct pmt {
    program_number: u16,
    version_number: u8,
    pcr_pid: u16,
}





struct TS {
    pat: pat,
    pmt: Vec<program_map>,
}


struct program {
    num: u16,
    id: u16,
}

struct program_map {
    program_num: u16,
    version: u8,
    streams: Vec<stream>,
}


struct stream {
    s_type:u8,
    s_pid: u16,
}

fn ts_parser(buffer: &[u8]) {

    let pid = ((buffer[1] as u16) << 8 | (buffer[2] as u16)) & 0x1FFF;
    if pid != 0 {
        return;
    }

    println!(
        "{:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X}", 
             buffer[0], 
             buffer[1], 
             buffer[2], 
             buffer[3], 
             buffer[4], 
             buffer[5], 
             buffer[6], 
             buffer[7], 
             buffer[8], 
             buffer[9], 
             buffer[10], 
             buffer[11]
             );
    println!("\tPID {:04X}", pid);




    let mut ts_adaptation_field_len:usize = 0;

    let mut transport_error_indicator = 0;
    if buffer[1] & 0x80 != 0 {
        transport_error_indicator = 1;
    }

    let mut payload_unit_start_indicator = 0;
    if buffer[1] & 0x40 != 0 {
        payload_unit_start_indicator = 1;
    }

    let transport_scrambling_control = (buffer[3] & 0xC0) >> 6;

    let adaptation_field_control = (buffer[3] & 0x30) >> 4;

    let continuity_counter = (buffer[3] & 0x0F);

    println!("\tpayload_unit_start_indicator 0b{:b}", payload_unit_start_indicator);
    println!("\tts_adaptation_field 0b{:02b}", adaptation_field_control);
    if adaptation_field_control == 0b10 || adaptation_field_control == 0b11 {
        ts_adaptation_field_len = ts_adaptation_field(&buffer[4..]);
        ts_adaptation_field_len += 1;
        println!("\tts_adaptation_field_len {}", ts_adaptation_field_len);
    }
    

    if pid == 0x0000 {
        let pat_tmp = pat_parser(&buffer[4..], payload_unit_start_indicator);
        for p in pat_tmp.programs.iter() {
            println!("\t\t{}\t{}", p.num, p.id);
        }
    }
}

fn ts_adaptation_field(buffer: &[u8]) -> usize {
    let len = buffer[0];
    return len as usize;
}

fn pat_parser(buffer: &[u8], indicator:u8) -> pat {
    let mut pointer_field = 0;
    let mut pat_buffer = &buffer[0..];
    if indicator != 0 {
        pointer_field = buffer[0];
        pat_buffer = &buffer[1..];
    }
    println!("pat_parser {}", pat_buffer.len());
    println!("\t table_id 0x{:02X}", pat_buffer[0]);

    println!("\t section_syntax_indicator 0b{:b}",(pat_buffer[1] & 0x80) >> 7);

    let section_length:usize = ((pat_buffer[1] as usize) << 8 | (pat_buffer[2] as usize)) & 0x0FFF;
    println!("\t section_length {}", section_length);


    let transport_stream_id = ((pat_buffer[3] as u16) << 8 | (pat_buffer[4] as u16));
    println!("\t transport_stream_id {}", transport_stream_id);

    let version = (pat_buffer[5] & 0x3E) >> 1;
    println!("\t version_number 0x{:X}", version);
    println!("\t current_next_indicator 0b{}",(pat_buffer[5] & 0x01));
    println!("\t section_number 0x{:X}",(pat_buffer[6]));
    println!("\t last_section_number 0x{:X}",(pat_buffer[7]));

    let program_num = (section_length - 5) / 4;
    println!("\t programs {}", program_num);
    println!("\t\t {}\t{}", "PNUM", "PID");

    let x:Vec<program>= Vec::new();
    let mut pat_vec = pat {
        version: version,
        programs: x,
    };
    for cnt in 0..program_num {
        let w = cnt * 4;
        let program_num = ((pat_buffer[8 + w] as u16) << 8) | (pat_buffer[9 + w] as u16);
        let program_id = ((pat_buffer[10 + w] as u16) << 8) | (pat_buffer[11 + w] as u16) & 0x1FFF;
        let pat_tmp = program {
            num: program_num,
            id: program_id
        };

        pat_vec.programs.push(pat_tmp);
        //println!("\t\t {:04X}\t{:04X}", program_num, program_id);
    }
    pat_vec
}

fn pmt_parser(buffer: &[u8]) {
    println!("pmt_parser {:X}",buffer[0]);
}


fn main() -> io::Result<()> {

    let args: Vec<String> = env::args().collect();
    let filename = match args.get(1) {
        Some(string) => string,
        None => {
            println!("HELP");
            println!("{} [TS FILE]", args[0]);
            return Ok(());
        },
    };
    let mut f = File::open(filename)?;

    let mut buffer = [0;188];

    let mut count = 0;

    loop {
        let n = f.read(&mut buffer[..])?;
        if n <= 0 {
            break;
        }

        if buffer[0] != 0x47 {
            println!("Sync Error: {}", buffer[0]);
            break;
        }

        ts_parser(&buffer);

        count += 1;
        if count == 10000 {
            break;
        }

        continue;

        /*
        for i in 0..n {
            print!("{:02X} ", &buffer[i]);
            if (i+1) % 16 == 0 {
                println!("");
            }
        }
        */

    }
    println!("\nCount {}", count);

    println!("");
    Ok(())
}
