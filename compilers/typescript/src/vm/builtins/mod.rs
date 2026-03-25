//! builtins 模块
use std::{collections::HashMap, rc::Rc};
use typescript_types::TsValue;

/// 内置函数和对象
#[derive(Debug, Clone)]
pub struct Builtins {
    /// console 对象
    pub console: HashMap<String, TsValue>,
    /// Math 对象
    pub math: HashMap<String, TsValue>,
    /// JSON 对象
    pub json: HashMap<String, TsValue>,
    /// Date 构造函数
    pub date_constructor: TsValue,
    /// RegExp 构造函数
    pub regexp_constructor: TsValue,
    /// Map 构造函数
    pub map_constructor: TsValue,
    /// Set 构造函数
    pub set_constructor: TsValue,
    /// Array 构造函数
    pub array_constructor: TsValue,
    /// Object 构造函数
    pub object_constructor: TsValue,
    /// String 构造函数
    pub string_constructor: TsValue,
    /// Number 构造函数
    pub number_constructor: TsValue,
    /// Boolean 构造函数
    pub boolean_constructor: TsValue,
    /// Symbol 构造函数
    pub symbol_constructor: TsValue,
    /// BigInt 构造函数
    pub bigint_constructor: TsValue,
}

impl Builtins {
    /// 创建内置对象
    pub fn new() -> Self {
        let mut console = HashMap::new();
        console.insert(
            "log".to_string(),
            TsValue::Function(Rc::new(|args| {
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        print!(" ");
                    }
                    print!("{}", arg.to_string());
                }
                println!();
                TsValue::Undefined
            })),
        );
        console.insert(
            "error".to_string(),
            TsValue::Function(Rc::new(|args| {
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        eprint!(" ");
                    }
                    eprint!("{}", arg.to_string());
                }
                eprintln!();
                TsValue::Undefined
            })),
        );
        console.insert(
            "warn".to_string(),
            TsValue::Function(Rc::new(|args| {
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        eprint!(" ");
                    }
                    eprint!("Warning: {}", arg.to_string());
                }
                eprintln!();
                TsValue::Undefined
            })),
        );
        console.insert(
            "info".to_string(),
            TsValue::Function(Rc::new(|args| {
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        print!(" ");
                    }
                    print!("Info: {}", arg.to_string());
                }
                println!();
                TsValue::Undefined
            })),
        );
        console.insert(
            "debug".to_string(),
            TsValue::Function(Rc::new(|args| {
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        print!(" ");
                    }
                    print!("Debug: {}", arg.to_string());
                }
                println!();
                TsValue::Undefined
            })),
        );
        console.insert(
            "trace".to_string(),
            TsValue::Function(Rc::new(|args| {
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        print!(" ");
                    }
                    print!("Trace: {}", arg.to_string());
                }
                println!();
                // 打印调用栈
                println!("Stack trace:");
                let backtrace = std::backtrace::Backtrace::capture();
                println!("{}", backtrace);
                TsValue::Undefined
            })),
        );
        console.insert(
            "table".to_string(),
            TsValue::Function(Rc::new(|args| {
                println!("Table:");
                for (i, arg) in args.iter().enumerate() {
                    println!("  [{}]: {}", i, arg.to_string());
                }
                TsValue::Undefined
            })),
        );
        console.insert(
            "time".to_string(),
            TsValue::Function(Rc::new(|args| {
                if let Some(TsValue::String(label)) = args.first() {
                    println!("Timer started: {}", label);
                }
                TsValue::Undefined
            })),
        );
        console.insert(
            "timeEnd".to_string(),
            TsValue::Function(Rc::new(|args| {
                if let Some(TsValue::String(label)) = args.first() {
                    println!("Timer ended: {}", label);
                }
                TsValue::Undefined
            })),
        );

        let mut math = HashMap::new();
        math.insert("PI".to_string(), TsValue::Number(std::f64::consts::PI));
        math.insert("E".to_string(), TsValue::Number(std::f64::consts::E));
        math.insert("LN2".to_string(), TsValue::Number(std::f64::consts::LN_2));
        math.insert("LN10".to_string(), TsValue::Number(std::f64::consts::LN_10));
        math.insert("LOG2E".to_string(), TsValue::Number(std::f64::consts::LOG2_E));
        math.insert("LOG10E".to_string(), TsValue::Number(std::f64::consts::LOG10_E));
        math.insert("SQRT1_2".to_string(), TsValue::Number(std::f64::consts::FRAC_1_SQRT_2));
        math.insert("SQRT2".to_string(), TsValue::Number(std::f64::consts::SQRT_2));
        math.insert(
            "abs".to_string(),
            TsValue::Function(Rc::new(|args| {
                if let Some(TsValue::Number(n)) = args.first() { TsValue::Number(n.abs()) } else { TsValue::Number(f64::NAN) }
            })),
        );
        math.insert(
            "acos".to_string(),
            TsValue::Function(Rc::new(|args| {
                if let Some(TsValue::Number(n)) = args.first() { TsValue::Number(n.acos()) } else { TsValue::Number(f64::NAN) }
            })),
        );
        math.insert(
            "asin".to_string(),
            TsValue::Function(Rc::new(|args| {
                if let Some(TsValue::Number(n)) = args.first() { TsValue::Number(n.asin()) } else { TsValue::Number(f64::NAN) }
            })),
        );
        math.insert(
            "atan".to_string(),
            TsValue::Function(Rc::new(|args| {
                if let Some(TsValue::Number(n)) = args.first() { TsValue::Number(n.atan()) } else { TsValue::Number(f64::NAN) }
            })),
        );
        math.insert(
            "atan2".to_string(),
            TsValue::Function(Rc::new(|args| {
                if let (Some(TsValue::Number(y)), Some(TsValue::Number(x))) = (args.get(0), args.get(1)) {
                    TsValue::Number(y.atan2(*x))
                }
                else {
                    TsValue::Number(f64::NAN)
                }
            })),
        );
        math.insert(
            "ceil".to_string(),
            TsValue::Function(Rc::new(|args| {
                if let Some(TsValue::Number(n)) = args.first() { TsValue::Number(n.ceil()) } else { TsValue::Number(f64::NAN) }
            })),
        );
        math.insert(
            "cos".to_string(),
            TsValue::Function(Rc::new(|args| {
                if let Some(TsValue::Number(n)) = args.first() { TsValue::Number(n.cos()) } else { TsValue::Number(f64::NAN) }
            })),
        );
        math.insert(
            "exp".to_string(),
            TsValue::Function(Rc::new(|args| {
                if let Some(TsValue::Number(n)) = args.first() { TsValue::Number(n.exp()) } else { TsValue::Number(f64::NAN) }
            })),
        );
        math.insert(
            "floor".to_string(),
            TsValue::Function(Rc::new(|args| {
                if let Some(TsValue::Number(n)) = args.first() { TsValue::Number(n.floor()) } else { TsValue::Number(f64::NAN) }
            })),
        );
        math.insert(
            "log".to_string(),
            TsValue::Function(Rc::new(|args| {
                if let Some(TsValue::Number(n)) = args.first() { TsValue::Number(n.ln()) } else { TsValue::Number(f64::NAN) }
            })),
        );
        math.insert(
            "max".to_string(),
            TsValue::Function(Rc::new(|args| {
                let mut max = f64::NEG_INFINITY;
                for arg in args.iter() {
                    if let TsValue::Number(n) = arg {
                        if *n > max {
                            max = *n;
                        }
                    }
                }
                TsValue::Number(max)
            })),
        );
        math.insert(
            "min".to_string(),
            TsValue::Function(Rc::new(|args| {
                let mut min = f64::INFINITY;
                for arg in args.iter() {
                    if let TsValue::Number(n) = arg {
                        if *n < min {
                            min = *n;
                        }
                    }
                }
                TsValue::Number(min)
            })),
        );
        math.insert(
            "pow".to_string(),
            TsValue::Function(Rc::new(|args| {
                if let (Some(TsValue::Number(base)), Some(TsValue::Number(exp))) = (args.get(0), args.get(1)) {
                    TsValue::Number(base.powf(*exp))
                }
                else {
                    TsValue::Number(f64::NAN)
                }
            })),
        );
        math.insert("random".to_string(), TsValue::Function(Rc::new(|_| TsValue::Number(rand::random()))));
        math.insert(
            "round".to_string(),
            TsValue::Function(Rc::new(|args| {
                if let Some(TsValue::Number(n)) = args.first() { TsValue::Number(n.round()) } else { TsValue::Number(f64::NAN) }
            })),
        );
        math.insert(
            "sin".to_string(),
            TsValue::Function(Rc::new(|args| {
                if let Some(TsValue::Number(n)) = args.first() { TsValue::Number(n.sin()) } else { TsValue::Number(f64::NAN) }
            })),
        );
        math.insert(
            "sqrt".to_string(),
            TsValue::Function(Rc::new(|args| {
                if let Some(TsValue::Number(n)) = args.first() { TsValue::Number(n.sqrt()) } else { TsValue::Number(f64::NAN) }
            })),
        );
        math.insert(
            "tan".to_string(),
            TsValue::Function(Rc::new(|args| {
                if let Some(TsValue::Number(n)) = args.first() { TsValue::Number(n.tan()) } else { TsValue::Number(f64::NAN) }
            })),
        );

        let mut json = HashMap::new();
        json.insert(
            "parse".to_string(),
            TsValue::Function(Rc::new(|args| {
                if let Some(TsValue::String(_s)) = args.first() {
                    // 简化的 JSON 解析
                    TsValue::Object(std::collections::HashMap::new())
                }
                else {
                    TsValue::Undefined
                }
            })),
        );
        json.insert(
            "stringify".to_string(),
            TsValue::Function(Rc::new(|args| {
                if let Some(arg) = args.first() { TsValue::String(arg.to_string()) } else { TsValue::Undefined }
            })),
        );

        Self {
            console,
            math,
            json,
            date_constructor: TsValue::Function(Rc::new(|args| {
                if let Some(TsValue::Number(_timestamp)) = args.first() {
                    TsValue::Object(std::collections::HashMap::new())
                }
                else {
                    TsValue::Object(std::collections::HashMap::new())
                }
            })),
            regexp_constructor: TsValue::Function(Rc::new(|args| {
                if let (Some(TsValue::String(_pattern)), Some(TsValue::String(_flags))) = (args.get(0), args.get(1)) {
                    TsValue::Object(std::collections::HashMap::new())
                }
                else if let Some(TsValue::String(_pattern)) = args.first() {
                    TsValue::Object(std::collections::HashMap::new())
                }
                else {
                    TsValue::Object(std::collections::HashMap::new())
                }
            })),
            map_constructor: TsValue::Function(Rc::new(|args| TsValue::Object(std::collections::HashMap::new()))),
            set_constructor: TsValue::Function(Rc::new(|args| TsValue::Object(std::collections::HashMap::new()))),
            array_constructor: TsValue::Function(Rc::new(|args| {
                if let Some(TsValue::Number(len)) = args.first() {
                    TsValue::Array(Vec::with_capacity(*len as usize))
                }
                else {
                    TsValue::Array(args.to_vec())
                }
            })),
            object_constructor: TsValue::Function(Rc::new(|args| {
                if let Some(TsValue::Object(obj)) = args.first() {
                    TsValue::Object(obj.clone())
                }
                else {
                    TsValue::Object(std::collections::HashMap::new())
                }
            })),
            string_constructor: TsValue::Function(Rc::new(|args| {
                if let Some(arg) = args.first() { TsValue::String(arg.to_string()) } else { TsValue::String(String::new()) }
            })),
            number_constructor: TsValue::Function(Rc::new(|args| {
                if let Some(arg) = args.first() {
                    if let TsValue::String(s) = arg {
                        TsValue::Number(s.parse().unwrap_or(f64::NAN))
                    }
                    else if let TsValue::Number(n) = arg {
                        TsValue::Number(*n)
                    }
                    else {
                        TsValue::Number(f64::NAN)
                    }
                }
                else {
                    TsValue::Number(0.0)
                }
            })),
            boolean_constructor: TsValue::Function(Rc::new(|args| {
                if let Some(arg) = args.first() { TsValue::Boolean(arg.to_boolean()) } else { TsValue::Boolean(false) }
            })),
            symbol_constructor: TsValue::Function(Rc::new(|args| {
                if let Some(TsValue::String(description)) = args.first() {
                    TsValue::Symbol(description.clone())
                }
                else {
                    TsValue::Symbol(String::new())
                }
            })),
            bigint_constructor: TsValue::Function(Rc::new(|args| {
                if let Some(arg) = args.first() {
                    if let TsValue::String(s) = arg {
                        if let Ok(n) = s.parse::<i128>() { TsValue::BigInt(n) } else { TsValue::BigInt(0) }
                    }
                    else if let TsValue::Number(n) = arg {
                        TsValue::BigInt(*n as i128)
                    }
                    else {
                        TsValue::BigInt(0)
                    }
                }
                else {
                    TsValue::BigInt(0)
                }
            })),
        }
    }

    /// 获取 console 对象
    pub fn console_object(&self) -> TsValue {
        TsValue::Object(self.console.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
    }

    /// 获取 Math 对象
    pub fn math_object(&self) -> TsValue {
        TsValue::Object(self.math.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
    }

    /// 获取 JSON 对象
    pub fn json_object(&self) -> TsValue {
        TsValue::Object(self.json.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
    }
}

impl Default for Builtins {
    fn default() -> Self {
        Self::new()
    }
}
