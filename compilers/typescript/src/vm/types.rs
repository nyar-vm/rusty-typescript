//! VM type definitions

use typescript_types::TsValue;

/// Call frame for function execution
#[derive(Debug, Clone)]
pub struct CallFrame {
    /// Function name
    pub function_name: String,
    /// Local variables
    pub locals: Vec<(String, TsValue)>,
    /// Return address (IP)
    pub return_address: usize,
    /// Return instruction pointer
    pub return_ip: usize,
}

impl CallFrame {
    /// Creates a new call frame
    pub fn new(function_name: String, return_address: usize) -> Self {
        Self {
            function_name,
            locals: Vec::new(),
            return_address,
            return_ip: return_address,
        }
    }

    /// Gets a local variable
    pub fn get_local(&self, name: &str) -> Option<&TsValue> {
        self.locals.iter().find(|(n, _)| n == name).map(|(_, v)| v)
    }

    /// Sets a local variable
    pub fn set_local(&mut self, name: String, value: TsValue) {
        if let Some((_, v)) = self.locals.iter_mut().find(|(n, _)| n == &name) {
            *v = value;
        } else {
            self.locals.push((name, value));
        }
    }
}

/// Exception handler
#[derive(Debug, Clone)]
pub struct ExceptionHandler {
    /// Handler address
    pub handler_address: usize,
    /// Stack depth at handler
    pub stack_depth: usize,
    /// Start IP of try block
    pub start_ip: usize,
    /// End IP of try block
    pub end_ip: usize,
    /// Handler IP
    pub handler_ip: usize,
    /// Exception variable name
    pub exception_var: Option<String>,
    /// Has finally block
    pub has_finally: bool,
    /// Finally IP
    pub finally_ip: Option<usize>,
}

/// Module instance
#[derive(Debug, Clone)]
pub struct ModuleInstance {
    /// Module name
    pub name: String,
    /// Exported values
    pub exports: Vec<(String, TsValue)>,
}

impl ModuleInstance {
    /// Creates a new module instance
    pub fn new(name: String) -> Self {
        Self {
            name,
            exports: Vec::new(),
        }
    }

    /// Exports a value from the module
    pub fn export(&mut self, name: String, value: TsValue) {
        self.exports.push((name, value));
    }
}

/// Function definition
#[derive(Debug, Clone)]
pub struct Function {
    /// Function name
    pub name: String,
    /// Parameter names
    pub params: Vec<String>,
    /// Function body instructions
    pub body: Vec<Instruction>,
}

/// Built-in functions and objects
#[derive(Debug, Clone)]
pub struct Builtins {
    /// Console object
    pub console: Vec<(String, TsValue)>,
}

impl Builtins {
    /// Creates new builtins
    pub fn new() -> Self {
        Self {
            console: Vec::new(),
        }
    }
}

/// Performance monitor
#[derive(Debug, Clone)]
pub struct PerformanceMonitor {
    /// Instruction count
    pub instruction_count: u64,
    /// Memory allocations
    pub memory_allocations: u64,
}

impl PerformanceMonitor {
    /// Creates a new performance monitor
    pub fn new() -> Self {
        Self {
            instruction_count: 0,
            memory_allocations: 0,
        }
    }

    /// Records an instruction execution
    pub fn record_instruction(&mut self) {
        self.instruction_count += 1;
    }

    /// Records a memory allocation
    pub fn record_memory_allocation(&mut self, _size: usize) {
        self.memory_allocations += 1;
    }

    /// Records a memory deallocation
    pub fn record_memory_deallocation(&mut self, _size: usize) {
        // Track deallocations if needed
    }
}

/// VM Instruction
#[derive(Debug, Clone)]
pub enum Instruction {
    /// Push a value onto the stack
    Push(TsValue),
    /// Push undefined onto the stack
    PushUndefined,
    /// Pop a value from the stack
    Pop,
    /// Load a variable
    LoadVariable(String),
    /// Store a variable
    StoreVariable(String),
    /// Create a function
    CreateFunction(String, u32),
    /// Create a class
    CreateClass(String),
    /// Add a method to a class
    AddMethod(String),
    /// Call a function
    Call(u32),
    /// Return from a function
    Return,
    /// Jump to an address
    Jump(u32),
    /// Jump if false
    JumpIfFalse(u32),
    /// Jump loop
    JumpLoop(u32),
    /// Binary operation
    BinaryOp(String),
    /// Unary operation
    UnaryOp(String),
    /// Get member
    GetMember,
    /// Set member
    SetMember,
    /// Get index
    GetIndex,
    /// Set index
    SetIndex,
    /// Create array
    CreateArray(u32),
    /// Create object
    CreateObject(u32),
    /// Throw exception
    Throw,
    /// Try block start
    TryStart(u32),
    /// Try block end
    TryEnd,
    /// Catch block
    Catch(u32),
    /// Finally block
    Finally,
}
