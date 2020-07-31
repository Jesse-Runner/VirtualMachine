// Jesse Runner
// Tuesday, March 3rd, 2020
// This program will read in a binary file and convert it into a set of instruction enums
// It will then calculate the result of said instructions and return the value

extern crate byteorder;

use self::byteorder::{ByteOrder,BigEndian,ReadBytesExt};
use std::slice::Iter;
use std::fs;
use std::env;
use std::convert::TryInto;

#[derive(Debug,Clone)]
pub struct State {
    pub halt: bool, //Has the machine halted?
    pub pc: u32, //The current program counter, a 32-bit unsigned integer
    pub fp: u32, //The current frame pointer
    pub stack: Vec<Val>, //The stack, with maximum size STACK_SIZE
    pub heap: Vec<Val>, //The heap
    pub program: Vec<Instr> //The program being executed, a list of instructions
}

pub trait fromBinary {
    fn from_binary(i: &mut Iter<u8>) -> Self;
}

impl fromBinary for i32 { // function to convert binary into i32
    fn from_binary(bytes: &mut Iter<u8>) -> Self{
        let mut v = vec![];
        v.push(*bytes.next().unwrap());
        v.push(*bytes.next().unwrap());
        v.push(*bytes.next().unwrap());
        v.push(*bytes.next().unwrap());
        let i = BigEndian::read_i32(&v);     
        return i;
    }
}
// function to convert our u32 values into big endian
impl fromBinary for u32 { // function to convert binary into U32
    fn from_binary(bytes: &mut Iter<u8>) -> Self{
        let mut v = vec![];
        v.push(*bytes.next().unwrap());
        v.push(*bytes.next().unwrap());
        v.push(*bytes.next().unwrap());
        v.push(*bytes.next().unwrap());
        let u = BigEndian::read_u32(&v);   
        return u;
    }
}


#[derive(Debug,Clone)]
pub enum Unop {
    Neg, //Boolean negation
}

impl fromBinary for Unop { // function to convert binary into Unop
    fn from_binary(bytes: &mut Iter<u8>) -> Self {
        match bytes.next().unwrap(){
            0b0000_0000 => Unop::Neg,
            _=> panic!("Bad!")
        }
    }
}

#[derive(Debug,Clone)]
pub enum Binop {
    Add, //i32 addition
    Mul, //i32 multiplication
    Sub, //i32 subtraction
    Div, //i32 division (raises an error on divide by zero)
    Lt,  //Returns true if one i32 is less than another, otherwise false
    Eq,  //Returns true if one i32 is equal another, otherwise false
}

impl fromBinary for Binop { // function to convert binary into Binops
    fn from_binary(bytes: &mut Iter<u8>) -> Self {
        match bytes.next().unwrap(){
            0b0000_0000 => Binop::Add,
            0b0000_0001 => Binop::Mul,
            0b0000_0010 => Binop::Sub,
            0b0000_0011 => Binop::Div,
            0b0000_0100 => Binop::Lt,
            0b0000_0101 => Binop::Eq,
            _=> panic!("Bad!")
        }
    }
}
type Address = usize; // used with Vaddr inside Val enum
#[derive(Debug,Clone,PartialEq)]
pub enum Val {
    Vunit,
    Vi32(i32),      //32-bit signed integers
    Vbool(bool),    //Booleans
    Vloc(u32),      //Stack or instruction locations
    Vundef,         //The undefined value
    Vsize(i32),     //Metadata for heap objects that span multiple values
    Vaddr(Address) //Pointers to heap locations
}

impl fromBinary for Val { // function to convert binary into Vals
    fn from_binary(bytes: &mut Iter<u8>) -> Self {
        match bytes.next().unwrap(){
            0b0000_0000 => Val::Vunit,
            0b0000_0001 => {
                let i = <i32 as fromBinary>::from_binary(bytes);
                return Val::Vi32(i);
            }
            0b0000_0100 => {
                let i = <u32 as fromBinary>::from_binary(bytes);
                return Val::Vloc(i);
            }
            0b0000_0010 => Val::Vbool(true),
            0b0000_0011 => Val::Vbool(false),
            0b0000_0101 => Val::Vundef,
            _=> panic!("Bad!")
        }
    }
}

#[derive(Debug,Clone)]
pub enum Instr {
    Push(Val),     //Push(v): Push value v onto the stack      // Push Label onto the stack
    Pop,           //Pop a value from the stack, discarding it
    Peek(u32),     //Peek(i): Push onto the stack the ith value from the top
    Unary(Unop),   //Unary(u): Apply u to the top value on the stack
    Binary(Binop), //Binary(b): Apply b to the top two values on the stack, replacing them with the result
    Swap,          //Swap the top two values
    Alloc,         //Allocate an array on the heap
    Set,           //Write to a heap-allocated array
    Get,           //Read from a heap-allocated array
    Var(u32),      //Var(i): Get the value at stack position fp+i
    Store(u32),    //Store(i): Store a value at stack position fp+i
    SetFrame(u32), //SetFrame(i): Set fp = s.stack.len() - i
    Call,          //Function call
    Ret,           //Function return
    Branch,        //Conditional jump
    Halt           //Halt the machine
 }

 impl fromBinary for Instr { // function to convert binary  into instructions 
    fn from_binary(bytes: &mut Iter<u8>) -> Self {
        match bytes.next().unwrap(){
            0b0000_0000 => Instr::Push(Val::from_binary(bytes)),
            0b0000_0001 => Instr::Pop, 
            0b0000_0010 => {
                let i = <u32 as fromBinary>::from_binary(bytes);
                return Instr::Peek(i);
            } 
            0b0000_0011 => Instr::Unary(Unop::from_binary(bytes)),
            0b0000_0100 => Instr::Binary(Binop::from_binary(bytes)),
            0b0000_0101 => Instr::Swap, 
            0b0000_0110 => Instr::Alloc,
            0b0000_0111 => Instr::Set,
            0b0000_1000 => Instr::Get,
            0b0000_1001 => {
                let i = <u32 as fromBinary>::from_binary(bytes);
                return Instr::Var(i);
            }  
            0b0000_1010 => {
                let i = <u32 as fromBinary>::from_binary(bytes);
                return Instr::Store(i);
            }  
            0b0000_1011 => {
                let i = <u32 as fromBinary>::from_binary(bytes);
                return Instr::SetFrame(i);
            } 
            0b0000_1100 => Instr::Call,
            0b0000_1101 => Instr::Ret,
            0b0000_1110 => Instr::Branch,
            0b0000_1111 => Instr::Halt,
            _=> panic!("Bad! No match for the binary given")
        }
    }
}

fn eval_unary(s: &mut State){ // function to negate a bool value at top of stack
    let stack_top = s.stack.pop().expect("No value to pop"); // grabs top stack value
    match stack_top{ // match statement to ensure top value is a vbool
        Val::Vbool(y) =>{
            match y{
                true => { // if value is true, negate it to false
                    let new_bool = false;
                    s.stack.push(Val::Vbool(new_bool));
                }
                false =>{ // if value is false, negate it to true 
                    let new_bool = true;
                    s.stack.push(Val::Vbool(new_bool));
                }
                _=> panic!("Invalid bool made it through") // if somehow a value other than T/F slips through
            }
        }
        _=> panic!("cant apply unary to non-bool") // panics if not a bool value
    }
}

fn eval_binary(b: Binop, s: &mut State){ // function that applies binary operator b to two Val's
        let e1 = s.stack.pop().expect("No top value to pop -- Binary");
        let e2 = s.stack.pop().expect("No secondary value to pop -- Binary");

        match b{
            Binop::Add => { // addition case for binary operator
                match e1 {
                    Val::Vi32(v1) =>{
                        match e2 {
                            Val::Vi32(v2) => {
                                s.stack.push(Val::Vi32(v1 + v2));
                            }
                            _=> panic!("Expected Vi32 -- Binary add")
                        }
                    }
                    _ => panic!("Expected Vi32 -- Binary add")
                }
            }
            Binop::Sub =>{ // subtraction case for binary operator
                match e1 {
                    Val::Vi32(v1) =>{
                        match e2 {
                            Val::Vi32(v2) =>  {
                                s.stack.push(Val::Vi32(v1 - v2));
                            }
                            _=> panic!("Expected Vi32 -- Binary sub")
                        }
                    }
                    _ => panic!("Expected Vi32 -- Binary sub")
                }
            }
            Binop::Mul =>{ // multiplication case for binary operator
                match e1 {
                    Val::Vi32(v1) =>{
                        match e2 {
                            Val::Vi32(v2) => {
                                s.stack.push(Val::Vi32(v1 * v2));
                            }
                            _=> panic!("Expected Vi32 -- Binary mul")
                        }
                    }
                    _ => panic!("Expected Vi32 -- Binary mul")
                }
            }
            Binop::Div =>{ // division case for binary operator
                match e1 {
                    Val::Vi32(v1) =>{
                        match e2 {
                            Val::Vi32(v2) => {
                                if v2 != 0{
                                    s.stack.push(Val::Vi32(v1 / v2));
                                }
                                else {
                                    panic!("Cannot divide by zero -- Binary Div");
                                }
                            }
                            _=> panic!("Expected Vi32 -- Binary div")
                        }
                    }
                    _ => panic!("Expected Vi32 -- Binary div")
                }
            }
            Binop::Lt => // less than case for binary operator
            {
                match e1 {
                    Val::Vi32(v1) =>{
                        match e2 {
                            Val::Vi32(v2) => {
                                   if v1 < v2 {
                                        s.stack.push(Val::Vbool(true));
                                   }
                                   else{
                                        s.stack.push(Val::Vbool(false));
                                   }
                            }
                            _=> panic!("Expected Vi32 -- Binary lt")
                        }
                    }
                    _ => panic!("Expected Vi32 -- Binary lt")
                }
            }
            Binop::Eq => { // equal to case for binary operator
                match e1 {
                    Val::Vi32(v1) =>{ 
                        match e2 {
                            Val::Vi32(v2) => {
                                   if v1 == v2 {
                                        s.stack.push(Val::Vbool(true));
                                   }
                                   else{
                                        s.stack.push(Val::Vbool(false));
                                   }
                            }
                            _=> panic!("Expected Vi32 -- Binary Eq")
                        }
                    }
                    _ => panic!("Expected Vi32 -- Binary Eq")
                }
            }
        }
}

fn eval_alloc(s: &mut State){ // function to allocate values onto stack
    let top_of_stack = s.stack.pop().expect("No value to pop"); 
    let second_top_value = s.stack.pop().expect("No secondary value to pop");

    match second_top_value{
        Val::Vi32(x) => { // if the value is an i32, continue
                s.heap.push(Val::Vsize(x)); // push metadata for array size
                let mut counter = 0;
                'heaploop: loop{ // pushes top of stack value onto heap until our counter is equal to second_top_val
                    if counter == x{break 'heaploop};
                    if s.heap.len() > 1024 {
                        panic!("Heap is out of bounds")
                    }
                    s.heap.push(top_of_stack.clone());
                    counter = counter + 1;
                }
                let x_as_usize = x as usize ; // convert oto u32 for ensuing subtraction
                let array_start = s.heap.len() - x_as_usize - 1; 
                s.stack.push(Val::Vaddr(array_start)); // push vaddr onto stack
        }
        _=> panic!("Expected Vi32") // anything else will cause a panic
    }
}

fn eval_set (s: &mut State){ // function to store value at heap address base + idx + 1
    let val_to_be_stored = s.stack.pop().expect("Nothing to be popped in Set function"); // val to be stored
    let idx = s.stack.pop().expect("Can't pop a second value in Set function"); // idx
    let addr = s.stack.pop().expect("Can't pop a third value from Set function"); // base 

     match idx{
             Val::Vi32(x) => { // idx must be i32
                     let idx_val = x;
                     match addr {
                         Val::Vaddr(y) => { // addr must be type Vaddr
                             let y_as_u32 = y as i32;
                             let index_in_heap = (idx_val + y_as_u32 + 1) as usize;
                             s.heap[index_in_heap] = val_to_be_stored; // store in heap at given index
                         }
                         _ => panic!("Invalid address location inside Set function") 
                     }
             }
             _ => panic!("Can't index a non i32 in Set function")
     }
}

fn eval_get(s: &mut State){ // push value contained at heap address base + idx + 1 onto the stack
    let top_of_stack = s.stack.pop().expect("Nothing to pop -- Get function"); // idx
    let secondary_top = s.stack.pop().expect("No secondary value to pop -- Get function"); // base
    
    match top_of_stack{
        Val::Vi32(x) =>{ // must be i32
                let idx = x as u32;
                match secondary_top{
                    Val::Vaddr(y) => { // must be vaddr
                        let base = y as u32;
                        let heap_loc = (idx + base + 1) as usize; // index on the heap
                        let stack_val = s.heap[heap_loc].clone(); //value to be stored
                        s.stack.push(stack_val);  // push onto stack
                    }
                    _=> panic!("Get requires a Vaddr as secondary stack location")
                }
        }
        _=> panic!("Get requires an i32 at top of stack")

    }
}

fn eval_ret(s: &mut State){ // function to grab return value, restore caller_fp and caller_pc, and remove unecessary variables
    let mut ret_val = Val::Vi32(0); // had to initalize to something so I could use in if statements
    let mut caller_pc_vloc = Val::Vloc(0); // had to initalize to something so I could use in if statements
    let mut caller_fp_vloc = Val::Vloc(0); //had to initalize to something so I could use in if statements
    let mut end = s.fp; // initalize a counter to the frame pointer 
    let mut counter = (s.stack.len() - 1) as u32; // counter to count down from stack.len() -1 to framepointer
    let stack_size = s.stack.len() ; // variable that keeps track of initial size of stack before pops

    'poploop: loop{ // loop that goes from counter to fp and decrements counter each iteration
        if counter == (stack_size - 3) as u32 {
            caller_fp_vloc = s.stack.pop().expect("No tertiary value to pop -- Ret function");
            if (s.stack.len() == 0){
                break 'poploop;
            }
            counter = counter - 1;
        }
        else if counter == (stack_size - 2) as u32 {
             caller_pc_vloc = s.stack.pop().expect("No secondary value to pop -- Ret function");
             counter = counter - 1;
        }
        else if  counter == (stack_size - 1) as u32 {
             ret_val = s.stack.pop().expect("No value to pop -- Ret function");
             counter = counter - 1;
        }
        else {
            if s.stack.len() != 0{
                s.stack.pop().expect("No value to pop -- Ret function");
            }
            if counter == end || counter < 0 {
                break 'poploop ;
            }
            counter = counter - 1;
        }
    }
    match caller_pc_vloc{ 
        Val::Vloc(x) =>{
            let caller_pc = x ;
            match caller_fp_vloc{
                Val::Vloc(y) => {
                        let caller_fp = y;
                        s.fp = caller_fp;
                        s.pc = caller_pc;
                        s.stack.push(ret_val);
                }
                _=> panic!("caller_fp must be vloc")
            }
        }
        _=> panic!("caller_pc must be vloc")
    }
}
fn eval_branch(s: &mut State){ // function to branch to given target if second value b on stack is true, otherwise do nothing
    let new_pc_loc = s.stack.pop().expect("Nothing to pop off stack -- Branch"); // target to be branched to
    let determine = s.stack.pop().expect("No secondary value to pop off the stack -- Branch") ; // vbool b which determines

    match new_pc_loc{
        Val::Vloc(target) => { // must be of type vloc
                match determine{
                    Val::Vbool(y) =>{ // must be of type vbool
                            match y{
                                true => s.pc = target,
                                false => ()
                            }
                    }
                    _=> panic!("Secondary value must be of type Vbool -- Branch")
                }
        }
        _=> panic!("Top value on stack must be a vloc -- Branch")
    }
}
fn evaluate (i: Instr, s: &mut State){ // function to evaluate the given instruction and match it with correct helper function / set of instructions
    match i {
         Instr::Push(x) => { // pushes value onto stack if stack isnt greater than 1024
            if(s.stack.len() > 1024){
                panic!("Stack size exceeded");
            }
            else{
                s.stack.push(x);
            }
         }
         Instr::Pop => { // removes top value on stack if stack is populated
                s.stack.pop().expect("Nothing to pop");
         }
         Instr::Peek(x) =>{ // copies the value at x'th location onto top of stack
                let convert_x_to_usize: usize = x.try_into().unwrap();
                let copy_at_ith = s.stack[convert_x_to_usize].clone();
                s.stack.push(copy_at_ith);
         }
         Instr::Unary(x) =>{ // negation operator applied to top value on stack
                eval_unary(s);
         }
         Instr::Binary(x) =>{ // calls binary helper function
                let result = eval_binary(x,s);
         }
         Instr::Swap => {  // swaps the top two values on the stack
                let top_value = s.stack.pop().expect("No value to pop"); // pop the top value
                let second_top_value = s.stack.pop().expect("No secondary value to pop"); // pop the second to top value
                s.stack.push(top_value); // push the top value back on so its now secondary top
                s.stack.push(second_top_value); // push the secondary top value on so its now top
         }
         Instr::Alloc => { // calls alloc helper function
                eval_alloc(s);
         }
        Instr::Set => { // calls set helper function
            eval_set(s);
         }
         Instr::Get => { // calls get helper function
                eval_get(s);
         }
         Instr::Var(x) => { // pushes onto stack the value at frame pointer + x 
                let index_to_find = s.fp + x;
                if index_to_find as usize > s.stack.len() - 1{
                        panic!("Var index out of bound");
                }
                let val_to_push = s.stack[(index_to_find as usize)].clone();
                s.stack.push(val_to_push);
         }
         Instr::Store(x) => { // overwrites the value at stack address frame pointer + 1 with top value on stack
                let stack_top = s.stack.pop().expect("No value to pop -- Store function"); // top value on the stack is popped so we know what it is
                let stack_top_clone = stack_top.clone();
    
                let index_to_be_overwritten = s.fp + x ; // points to the index we're overwriting with top stack clone value
                if index_to_be_overwritten > s.stack.len() as u32 - 1 {
                    panic!("Store is going out of bounds");                
                }
                let index_to_be_overwritten_usize = index_to_be_overwritten as usize; // convert to usize so we can index
                s.stack[index_to_be_overwritten_usize] = stack_top_clone;
         }
        Instr::SetFrame(x) => { // sets the frame pointer according to given argument 
            s.stack.push(Val::Vloc(s.fp));
            let x_as_usize = x as usize;
            s.fp = (s.stack.len() - x_as_usize - 1) as u32;
        }
        Instr::Call => { // jumps to instructions at vloc on top of stack
            let x = s.stack.pop().unwrap(); 
            match x {
                Val::Vloc(a) => { // top of stack value must be vloc
                    s.stack.push(Val::Vloc(s.pc));
                    s.pc = a;
                }
                _ => panic!("Illegal use of call function") // if top value on stack isnt a vloc
            }
         }
        Instr::Ret => {  // calls ret helper function
                eval_ret(s);

         }
         Instr::Branch => { // calls branch helper function
                eval_branch(s);
         }
         Instr::Halt => { // gives the state the flag to halt the program
            s.halt = true;
         }
        _ => {
            ();
        }
    }
}

fn exec(s: &mut State, instruc_vec: Vec<Instr>){  // function to execute the main loop of our program
    let mut counter = 0;
    'mainloop: loop{ // loop to iterate through every instruction in our program
        if s.halt { break 'mainloop } // check to see if program has been given the halt signal, if so exit
        let pc = s.pc; // setting the program counter 
        s.pc = pc + 1; // setting the state's program counter to next instruction 
        if pc >= TryInto::<u32>::try_into(s.program.len()).unwrap(){ // checks to ensure pc isnt out of bounds
            panic!("exec: pc out of bounds")
        }
        let pcusize = pc as usize;
        let i = s.program[pcusize].clone();
        let newi = i.clone();

        println!("{}",i);
        for x in s.stack.clone(){
            println!("{:?}",x);
        }
        println!("{}","-------------");

        evaluate(i, s); // sends current instruction and state into evaluate function
     }
    // let result = s.stack.pop().unwrap();
    // print!("{:?}",result);
}

fn main() {
    let args: Vec<String> = env::args().collect(); // collects command line argument of filename
    let query = args[1].clone(); // query holds the command line argument
    let binaryvec = fs::read(query).expect("Wrong file"); // reads in our file into a binary vector 
    let mut iter = binaryvec.iter(); // iterator to traverse our binary vector 
    let buffer = <u32 as fromBinary>::from_binary(iter.by_ref()); // buffer to hold instructions
    let mut instruc_vec: Vec<Instr> = Vec::new(); // initalize our instruction vector
    for i in 0..buffer { // loop through our buffer and push each instruction into our vector 
        instruc_vec.push(Instr::from_binary(iter.by_ref())); 
    }
    let mut s: State = State{halt: false,  pc: 0, fp: 0, stack: Vec::<Val>::with_capacity(0), // initalize our state
                         heap: Vec::<Val>::with_capacity(0), program: instruc_vec.clone()};

    exec(&mut s, instruc_vec); // call our execution loop on our state and instruction vector 
}
