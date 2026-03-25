//! builtins 模块
//!
use std::collections::HashMap;
use std::rc::Rc;
use typescript_types::TsValue;

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
            "floor".to_string(),
            TsValue::Function(Rc::new(|args| {
                if let Some(TsValue::Number(n)) = args.first() { TsValue::Number(n.floor()) } else { TsValue::Number(f64::NAN) }
            })),
        );
        math.insert(
            "ceil".to_string(),
            TsValue::Function(Rc::new(|args| {
                if let Some(TsValue::Number(n)) = args.first() { TsValue::Number(n.ceil()) } else { TsValue::Number(f64::NAN) }
            })),
        );
        math.insert(
            "round".to_string(),
            TsValue::Function(Rc::new(|args| {
                if let Some(TsValue::Number(n)) = args.first() { TsValue::Number(n.round()) } else { TsValue::Number(f64::NAN) }
            })),
        );
        math.insert(
            "sqrt".to_string(),
            TsValue::Function(Rc::new(|args| {
                if let Some(TsValue::Number(n)) = args.first() { TsValue::Number(n.sqrt()) } else { TsValue::Number(f64::NAN) }
            })),
        );
        math.insert(
            "cbrt".to_string(),
            TsValue::Function(Rc::new(|args| {
                if let Some(TsValue::Number(n)) = args.first() { TsValue::Number(n.cbrt()) } else { TsValue::Number(f64::NAN) }
            })),
        );
        math.insert(
            "max".to_string(),
            TsValue::Function(Rc::new(|args| {
                let max = args
                    .iter()
                    .filter_map(|v| match v {
                        TsValue::Number(n) => Some(*n),
                        _ => None,
                    })
                    .fold(f64::NEG_INFINITY, f64::max);
                TsValue::Number(max)
            })),
        );
        math.insert(
            "min".to_string(),
            TsValue::Function(Rc::new(|args| {
                let min = args
                    .iter()
                    .filter_map(|v| match v {
                        TsValue::Number(n) => Some(*n),
                        _ => None,
                    })
                    .fold(f64::INFINITY, f64::min);
                TsValue::Number(min)
            })),
        );
        math.insert(
            "pow".to_string(),
            TsValue::Function(Rc::new(|args| {
                if let (Some(TsValue::Number(base)), Some(TsValue::Number(exp))) = (args.get(0), args.get(1)) {
                    TsValue::Number(base.powf(*exp))
                } else {
                    TsValue::Number(f64::NAN)
                }
            })),
        );
        math.insert(
            "exp".to_string(),
            TsValue::Function(Rc::new(|args| {
                if let Some(TsValue::Number(n)) = args.first() { TsValue::Number(n.exp()) } else { TsValue::Number(f64::NAN) }
            })),
        );
        math.insert(
            "log".to_string(),
            TsValue::Function(Rc::new(|args| {
                if let Some(TsValue::Number(n)) = args.first() { TsValue::Number(n.ln()) } else { TsValue::Number(f64::NAN) }
            })),
        );
        math.insert(
            "log10".to_string(),
            TsValue::Function(Rc::new(|args| {
                if let Some(TsValue::Number(n)) = args.first() { TsValue::Number(n.log10()) } else { TsValue::Number(f64::NAN) }
            })),
        );
        math.insert(
            "log2".to_string(),
            TsValue::Function(Rc::new(|args| {
                if let Some(TsValue::Number(n)) = args.first() { TsValue::Number(n.log2()) } else { TsValue::Number(f64::NAN) }
            })),
        );
        math.insert(
            "sin".to_string(),
            TsValue::Function(Rc::new(|args| {
                if let Some(TsValue::Number(n)) = args.first() { TsValue::Number(n.sin()) } else { TsValue::Number(f64::NAN) }
            })),
        );
        math.insert(
            "cos".to_string(),
            TsValue::Function(Rc::new(|args| {
                if let Some(TsValue::Number(n)) = args.first() { TsValue::Number(n.cos()) } else { TsValue::Number(f64::NAN) }
            })),
        );
        math.insert(
            "tan".to_string(),
            TsValue::Function(Rc::new(|args| {
                if let Some(TsValue::Number(n)) = args.first() { TsValue::Number(n.tan()) } else { TsValue::Number(f64::NAN) }
            })),
        );
        math.insert(
            "asin".to_string(),
            TsValue::Function(Rc::new(|args| {
                if let Some(TsValue::Number(n)) = args.first() { TsValue::Number(n.asin()) } else { TsValue::Number(f64::NAN) }
            })),
        );
        math.insert(
            "acos".to_string(),
            TsValue::Function(Rc::new(|args| {
                if let Some(TsValue::Number(n)) = args.first() { TsValue::Number(n.acos()) } else { TsValue::Number(f64::NAN) }
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
                } else {
                    TsValue::Number(f64::NAN)
                }
            })),
        );
        math.insert(
            "sinh".to_string(),
            TsValue::Function(Rc::new(|args| {
                if let Some(TsValue::Number(n)) = args.first() { TsValue::Number(n.sinh()) } else { TsValue::Number(f64::NAN) }
            })),
        );
        math.insert(
            "cosh".to_string(),
            TsValue::Function(Rc::new(|args| {
                if let Some(TsValue::Number(n)) = args.first() { TsValue::Number(n.cosh()) } else { TsValue::Number(f64::NAN) }
            })),
        );
        math.insert(
            "tanh".to_string(),
            TsValue::Function(Rc::new(|args| {
                if let Some(TsValue::Number(n)) = args.first() { TsValue::Number(n.tanh()) } else { TsValue::Number(f64::NAN) }
            })),
        );
        math.insert("random".to_string(), TsValue::Function(Rc::new(|_| TsValue::Number(rand::random::<f64>()))));

        let mut json = HashMap::new();
        json.insert(
            "parse".to_string(),
            TsValue::Function(Rc::new(|args| {
                if let Some(TsValue::String(s)) = args.first() {
                    // 简化实现，实际应该解析 JSON
                    TsValue::Object(vec![("parsed".to_string(), TsValue::String(s.clone()))])
                }
                else {
                    TsValue::Error("JSON.parse expects a string".to_string())
                }
            })),
        );
        json.insert(
            "stringify".to_string(),
            TsValue::Function(Rc::new(|args| {
                if let Some(value) = args.first() { TsValue::String(value.to_string()) } else { TsValue::Undefined }
            })),
        );

        Self {
            console,
            math,
            json,
            date_constructor: TsValue::Function(Rc::new(|args| {
                if let Some(TsValue::Number(timestamp)) = args.first() {
                    TsValue::Date(*timestamp as i64)
                } else {
                    TsValue::Date(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64)
                }
            })),
            regexp_constructor: TsValue::Function(Rc::new(|args| {
                if let Some(TsValue::String(pattern)) = args.first() {
                    TsValue::RegExp(pattern.clone())
                } else {
                    TsValue::RegExp(String::new())
                }
            })),
            map_constructor: TsValue::Function(Rc::new(|args| {
                TsValue::Map(vec![])
            })),
            set_constructor: TsValue::Function(Rc::new(|args| {
                TsValue::Set(vec![])
            })),
            array_constructor: TsValue::Function(Rc::new(|args| TsValue::Array(args.to_vec()))),
            object_constructor: TsValue::Function(Rc::new(|args| {
                if let Some(TsValue::Object(props)) = args.first() {
                    TsValue::Object(props.clone())
                }
                else {
                    TsValue::Object(vec![])
                }
            })),
            string_constructor: TsValue::Function(Rc::new(|args| {
                if let Some(arg) = args.first() { TsValue::String(arg.to_string()) } else { TsValue::String(String::new()) }
            })),
            number_constructor: TsValue::Function(Rc::new(|args| {
                if let Some(arg) = args.first() { TsValue::Number(arg.to_number()) } else { TsValue::Number(0.0) }
            })),
            boolean_constructor: TsValue::Function(Rc::new(|args| {
                if let Some(arg) = args.first() { TsValue::Boolean(arg.to_boolean()) } else { TsValue::Boolean(false) }
            })),
            symbol_constructor: TsValue::Function(Rc::new(|args| {
                if let Some(TsValue::String(desc)) = args.first() {
                    TsValue::Symbol(desc.clone())
                } else {
                    TsValue::Symbol(String::new())
                }
            })),
            bigint_constructor: TsValue::Function(Rc::new(|args| {
                if let Some(TsValue::Number(n)) = args.first() {
                    TsValue::BigInt(*n as i128)
                } else if let Some(TsValue::String(s)) = args.first() {
                    if let Ok(bi) = s.parse::<i128>() {
                        TsValue::BigInt(bi)
                    } else {
                        TsValue::BigInt(0)
                    }
                } else {
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

/// 性能监控
#[derive(Debug, Clone)]