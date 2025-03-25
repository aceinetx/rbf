extern crate colorful;

use std::{env, io, process};
use std::collections::HashMap;
use std::io::Write;
use colorful::Colorful;
use colorful::RGB;
use std::fs;
use std::thread::sleep;
use std::time::Duration;
use hlua::Lua;
use once_cell::sync::Lazy;
static mut REPL_MODE: bool = false;
static mut llua: Lazy<Lua> = Lazy::new(|| Lua::new());
static mut custom_commands: Lazy<HashMap<String, String>> = Lazy::new(|| HashMap::new());
static mut custom_instructions: Lazy<HashMap<char, String>> = Lazy::new(|| HashMap::new());
static mut FILE_SOURCE: &str = "stdin";
static mut BF_MEMORY: Vec<i32> = vec![];
static mut BF_JUMPSTACK: Vec<i32> = vec![];
static mut POINTER_CELL: i32 = 0;
static mut MEMORY_CELLS: i32 = 30000;
static mut USED_MEMORY_CELLS_INDEXES: Vec<i32> = vec![];
static mut NOBOUNDS: bool = false;

static mut SKIP_NULL_LOOP: bool = true;
static mut SKIP_LOOP: bool = false;

fn rgb_print(text: &str, _r: u8, _g: u8, _b: u8){
    let txt = format!("{text}").color(RGB::new(_r, _g, _b));
    print!("{txt}");
}

fn rgb_println(text: &str, _r: u8, _g: u8, _b: u8){
    let txt = format!("{text}").color(RGB::new(_r, _g, _b));
    println!("{txt}");
}

fn raise_error(name: &str, description: &str){
    rgb_println(format!("\nError: {name}.{description}").as_str(), 255, 0, 0);
}

fn clamp(value: i32, min: i32, max: i32) -> i32{
    if value < min{
        return min;
    }
    if value > max {
        return max;
    }
    return value;
}

fn check_ptr_cell(){
    unsafe {
        if NOBOUNDS { return; }
        if POINTER_CELL < 0 {
            POINTER_CELL = 0;
            raise_error("MemError", "Cursor outside of memory boundaries (underflow)");
            if !REPL_MODE { process::exit(1) }
        } else if POINTER_CELL > MEMORY_CELLS {
            POINTER_CELL = MEMORY_CELLS;
            raise_error("MemError", "Cursor outside of memory boundaries (overflow)");
            if !REPL_MODE { process::exit(1) }
        }
    }
}

fn check_ptr_cell_with_source(_code: String, _index: i32, _line: Option<i32>){
    unsafe {
        if NOBOUNDS { return; }
        let line = _line.unwrap_or(0);
        if POINTER_CELL < 0 {
            POINTER_CELL = 0;
            raise_error("MemError", "Cursor outside of memory boundaries (underflow)");
            let range = 8;
            let idx_back = clamp(_index-range, 0, _code.len().try_into().unwrap());
            let idx_forward = clamp(_index+(range-1), 0, _code.len().try_into().unwrap());

            let line_str = format!("{} | ", (line+1));
            let mut cells_radius = String::from("");

            for _i in line_str.chars(){
                print!(" ");
            }

            for idx in idx_back..idx_forward{
                cells_radius.push(_code.chars().collect::<Vec<_>>()[idx as usize]);
                if idx == _index-1 {
                    rgb_print("┌─ occured here", 239,76,60);
                } else {
                    print!(" ");
                }
            }

            println!();

            println!("{line_str}{cells_radius}");

            if !REPL_MODE { process::exit(1) }
        } else if POINTER_CELL > MEMORY_CELLS-1 {
            POINTER_CELL = MEMORY_CELLS-1;
            raise_error("MemError", "Cursor outside of memory boundaries (overflow)");
            let range = 8;
            let idx_back = clamp(_index-range, 0, _code.len().try_into().unwrap());
            let idx_forward = clamp(_index+(range-1), 0, _code.len().try_into().unwrap());

            let line_str = format!("{} | ", (line+1));
            let mut cells_radius = String::from("");

            for _i in line_str.chars(){
                print!(" ");
            }

            for idx in idx_back..idx_forward{
                cells_radius.push(_code.chars().collect::<Vec<_>>()[idx as usize]);
                if idx == _index-1 {
                    rgb_print("┌─ occured here", 239,76,60);
                } else {
                    print!(" ");
                }
            }

            println!();

            println!("{line_str}{cells_radius}");

            if !REPL_MODE { process::exit(1) }
        }
    }
}

fn move_pointer(move_steps: i32){
    unsafe {
        POINTER_CELL += move_steps;
    }
}

fn write_memory(pointer_index: i32, value: i32){
    unsafe {
        BF_MEMORY[clamp(pointer_index, 0, MEMORY_CELLS-1) as usize] = (value as u8) as i32;
    }
}

fn read_memory(pointer_index: i32) -> i32{
    unsafe {
        return (BF_MEMORY[clamp(pointer_index, 0, MEMORY_CELLS-1) as usize] as u8) as i32;
    }
}

fn bf_clean_code(mut code: &mut String){
    unsafe {
        if custom_instructions.is_empty() {
            let mut new_code = String::from("");
            for c in code.chars() {
                if String::from(",.<>[]*+-").contains(c) {
                    new_code.push(c);
                }
            }
            *code = new_code;
        }
    }
}

fn bf_evaluate(code: String, line: Option<i32>) -> u8{
    let ln = line.unwrap_or(0);
    let mut idx = 0;
    while idx < code.len(){
        let c = code.chars().collect::<Vec<_>>()[idx];
        unsafe {

            if SKIP_LOOP{
                if c != ']' {
                    idx += 1;
                    continue;
                }
            }

            match c {
                '+' => {
                    let current_value = read_memory(POINTER_CELL);
                    write_memory(POINTER_CELL, current_value+1);
                    if !USED_MEMORY_CELLS_INDEXES.contains(&POINTER_CELL) {
                        USED_MEMORY_CELLS_INDEXES.push(POINTER_CELL);
                    }
                }

                '-' => {
                    let current_value = read_memory(POINTER_CELL);
                    write_memory(POINTER_CELL, current_value-1);
                    if !USED_MEMORY_CELLS_INDEXES.contains(&POINTER_CELL) {
                        USED_MEMORY_CELLS_INDEXES.push(POINTER_CELL);
                    }
                }

                '>' => {
                    POINTER_CELL += 1;
                }

                '<' => {
                    POINTER_CELL -= 1;
                }

                '.' => {
                    let current_value = read_memory(POINTER_CELL) as u8;
                    let buf = String::from(current_value as char);
                    print!("{buf}");
                    let _result_flush_error = io::stdout().flush();
                }

                ',' => {
                    let mut string_in = String::new();
                    io::stdin().read_line(&mut string_in).unwrap();

                    write_memory(POINTER_CELL, string_in.chars().collect::<Vec<_>>()[0] as i32);
                }

                '*' => {
                    let current_value = read_memory(POINTER_CELL) as u8;
                    let buf = String::from(current_value.to_string());
                    print!("{buf}");
                    let _result_flush_error = io::stdout().flush();
                }

                '[' => {
                    let current_value = read_memory(POINTER_CELL) as u8;
                    if current_value != 0 {
                        BF_JUMPSTACK.push(idx.clone() as i32);
                    } else {
                        if SKIP_NULL_LOOP {
                            SKIP_LOOP = true;
                        }
                    }
                }

                ']' => {

                    if SKIP_LOOP == true{
                        SKIP_LOOP = false;
                        idx += 1;
                        continue;
                    }

                    if BF_JUMPSTACK.len() > 0 {
                        let current_value = read_memory(POINTER_CELL);
                        if current_value > 0{
                            idx = BF_JUMPSTACK[BF_JUMPSTACK.len() - 1] as usize;
                        } else {
                            BF_JUMPSTACK.pop();
                        }
                    }
                }

                _ => {
                }
            }
            for (inst, funcname) in custom_instructions.clone(){
                if inst == c {
                    let mut func: hlua::LuaFunction<_> = llua.get(funcname).unwrap();
                    let _ = func.call::<i32>();
                }
            }
        }
        idx += 1;
        check_ptr_cell_with_source(code.clone(), (idx as usize).try_into().unwrap(), Some(ln));
    }

    return 0;
}

fn reclear_memory(){
    unsafe {
        BF_MEMORY.clear();

        for _i in 0..MEMORY_CELLS{
                BF_MEMORY.push(0);
            }
    }
}

fn execute_script(){
    // scripting api
    unsafe {
        let contents = fs::read_to_string("rbf.lua");
        if !contents.is_err() {
            //let mut lua = Lua::new();
            //llua = Lazy::new(|| Lua::new());
            let mut lua = &mut llua;
            lua.openlibs();
            // pre-built functions
            
            lua.set("warn", hlua::function1(move |x: String| -> i32 {
                rgb_print(format!("{}", x).as_str(), 252, 140, 3);
                0
            }));

            lua.set("err", hlua::function1(move |x: String| -> i32 {
                rgb_print(format!("{}", x).as_str(), 255, 0, 0);
                0
            }));

            lua.set("print_rgb", hlua::function4(move |x: String, r: i32, g: i32, b: i32| -> i32 {
                rgb_print(format!("{}", x).as_str(), r.try_into().unwrap(), g.try_into().unwrap(), b.try_into().unwrap());
                0
            }));

            lua.set("wait", hlua::function1(move |delay: f64| -> i32 {
                sleep(Duration::new((delay as u64), 0));
                0
            }));

            lua.set("exit", hlua::function1(move |code: i32| -> i32 {
                process::exit(code);
                0
            }));

            lua.set("rbf_exec", hlua::function1(move |x: String| -> i32 {
                if BF_MEMORY.is_empty() {
                    reclear_memory();
                }
                let mut buf = x.clone();
                bf_clean_code(&mut buf);
                bf_evaluate(buf, Some(0));

                0
            }));

            lua.set("rbf_exec", hlua::function1(move |x: String| -> i32 {
                if BF_MEMORY.is_empty() {
                    reclear_memory();
                }
                let mut buf = x.clone();
                bf_clean_code(&mut buf);
                bf_evaluate(buf, Some(0));

                0
            }));

            lua.set("rbf_customcmd", hlua::function2(move |cmd: String, funcname: String| -> i32 {
                custom_commands.insert(format!("!{cmd}"), funcname);

                0
            }));

            lua.set("rbf_custominstruction", hlua::function2(move |inst: String, funcname: String| -> i32 {
                custom_instructions.insert(inst.chars().collect::<Vec<_>>()[0], funcname);

                0
            }));

            lua.set("rbf_setptr", hlua::function1(move |x: i32| -> i32 {-
                //POINTER_CELL = x;

                0
            }));

            lua.set("rbf_writemem", hlua::function2(move |x: i32, v: i32| -> i32 {
                write_memory(x, v);

                0
            }));

            lua.set("rbf_readmem", hlua::function1(move |x: i32| -> i32 {
                return read_memory(x);
            }));

            lua.set("rbf_setmemlen", hlua::function1(move |x: i32| -> i32 {
                MEMORY_CELLS = x;
                reclear_memory();

                0
            }));

            lua.set("rbf_setnobounds", hlua::function1(move |x: bool| -> i32 {
                NOBOUNDS = x;

                0
            }));

            lua.set("rbf_getmem", hlua::function0(move || -> Vec<i32> {
                return BF_MEMORY.clone();
            }));

            lua.set("rbf_getptr", hlua::function0(move || -> i32 {
                return POINTER_CELL.clone();
            }));

            lua.set("throw", hlua::function1(move |x: String| -> i32 {
                raise_error("ScriptingError:LuaError", format!("{}", x).as_str());

                0
            }));

            let _code = contents.unwrap();
            let result = lua.execute::<()>(&_code);
            if result.is_err() {
                raise_error("ScriptingError:LuaError", format!("{}", result.err().unwrap()).as_str())
            }

            /*

            if lua.get::<i32, &str>("rbf_memlen") != None {
                let memcells: i32 = lua.get("rbf_memlen").unwrap();
                if memcells > 0 {
                    MEMORY_CELLS = memcells;
                    reclear_memory();
                } else {
                    raise_error("ScriptingError", "rbf_memlen is less than 1")
                }

            }

            if lua.get::<i32, &str>("rbf_nobounds") != None {
                let nob: i32 = lua.get("rbf_nobounds").unwrap();
                if [1, 0].contains(&nob) {
                    NOBOUNDS = nob != 0;
                } else {
                    raise_error("ScriptingError", "rbf_nobounds is not 1 or 0")
                }
            }

            if lua.get::<i32, &str>("rbf_ptr") != None {
                let p: i32 = lua.get("rbf_ptr").unwrap();
                if p >= 0 {
                    POINTER_CELL = p;
                } else {
                    raise_error("ScriptingError", "rbf_ptr is less than 0")
                }

            }*/
            rgb_println("rbf.lua has been reloaded!", 0, 255, 0);
        } else {
            rgb_println("Unable to read scripting file (rbf.lua), skipping", 3, 252, 202);
        }
    }
}

fn main(){

    let args: Vec<String> = env::args().collect();



    execute_script();

    reclear_memory();

    if args.len() < 2 { // Repl mode
        unsafe {
            REPL_MODE = true;
            rgb_println(format!("rBF v1.0 (repl, memlen: {MEMORY_CELLS}) !cmds - cmd list").as_str(), 3, 252, 202);
        }

        let mut l = 0;
        loop {
            let mut string_in = String::from("");

            unsafe { print!("{POINTER_CELL} >> "); }
            let _result_flush_error = io::stdout().flush();

            io::stdin().read_line(&mut string_in).unwrap();

            string_in = string_in.replace("\n", "");
            string_in = string_in.replace("\r", "");

            if string_in.eq("!q") {
                break;
            } else if string_in.eq("!cmds") {
                rgb_println("Command list", 0, 255, 0);
                rgb_println("!meminspect - view used memory", 3, 252, 202);
                rgb_println("!memclear - clear memory", 3, 252, 202);
                rgb_println("!reload - reload lua script", 3, 252, 202);
                rgb_println("!q - quit repl", 3, 252, 202);
            } else if string_in.eq("!meminspect") {
                unsafe {
                    let current_value = read_memory(POINTER_CELL) as u8;

                    rgb_println("Memory inspect:", 0, 255, 0);
                    rgb_println(format!("Pointer value: {current_value}").as_str(), 3, 252, 202);

                    for mem_cell in &USED_MEMORY_CELLS_INDEXES {
                        let ptr_value = read_memory(*mem_cell) as u8;

                        rgb_print(format!("└─ {mem_cell} │ ").as_str(), 115, 108, 237);
                        rgb_print(format!("{ptr_value}").as_str(), 100, 182, 172);
                        rgb_print(format!(" ( {} )", ptr_value as char).as_str(), 100, 182, 172);
                        println!();
                    }
                }
            } else if string_in.eq("!memclear") {
                reclear_memory();
            } else if string_in.eq("!reload") {
                unsafe {
                    llua = Lazy::new(|| Lua::new());
                    custom_commands.clear();
                    custom_instructions.clear();
                }
                execute_script();
            } else {
                let mut func_called = false;
                unsafe {
                    for (cmd, funcname) in custom_commands.clone() {
                        if string_in.eq(&cmd){
                            let mut func: hlua::LuaFunction<_> = llua.get(funcname).unwrap();
                            let _ = func.call::<i32>();
                            func_called = true;
                        }
                    }
                }

                if !func_called {
                    bf_clean_code(&mut string_in);
                    let _res = bf_evaluate(string_in.clone(), Some(l));
                    l += 1;
                }
            }
        }
    } else { // Normal mode
        let contents = fs::read_to_string(args[1].clone());

        if !contents.is_err() {
            let _code = contents.unwrap();
            let mut l = 0;
            for line in _code.split("\n"){
                let mut stringline = line.to_string();
                bf_clean_code(&mut stringline);
                let _res = bf_evaluate(stringline.clone(), Some(l));
                l += 1;
            }
        } else {
            raise_error("FileIOError", "Unable to read file")
        }
    }

}