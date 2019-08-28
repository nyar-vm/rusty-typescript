use typescript_ir::{Expression, PrimitiveType, Program, Statement, TypeAnnotation};
use typescript_types::TsError;

/// 类型环境
struct TypeEnvironment {
    /// 变量类型映射
    variables: std::collections::HashMap<String, TypeAnnotation>,
    /// 函数类型映射
    functions: std::collections::HashMap<String, (Vec<TypeAnnotation>, Option<TypeAnnotation>)>,
    /// 类型别名映射
    type_aliases: std::collections::HashMap<String, TypeAnnotation>,
    /// 接口映射
    interfaces: std::collections::HashMap<String, Vec<(String, TypeAnnotation, bool)>>,
}

impl Clone for TypeEnvironment {
    fn clone(&self) -> Self {
        Self {
            variables: self.variables.clone(),
            functions: self.functions.clone(),
            type_aliases: self.type_aliases.clone(),
            interfaces: self.interfaces.clone(),
        }
    }
}

impl TypeEnvironment {
    /// 创建一个新的类型环境
    fn new() -> Self {
        Self {
            variables: std::collections::HashMap::new(),
            functions: std::collections::HashMap::new(),
            type_aliases: std::collections::HashMap::new(),
            interfaces: std::collections::HashMap::new(),
        }
    }

    /// 添加变量类型
    fn add_variable(&mut self, name: String, ty: TypeAnnotation) {
        self.variables.insert(name, ty);
    }

    /// 添加函数类型
    fn add_function(&mut self, name: String, params: Vec<TypeAnnotation>, return_type: Option<TypeAnnotation>) {
        self.functions.insert(name, (params, return_type));
    }

    /// 添加类型别名
    fn add_type_alias(&mut self, name: String, ty: TypeAnnotation) {
        self.type_aliases.insert(name, ty);
    }

    /// 添加接口
    fn add_interface(&mut self, name: String, members: Vec<(String, TypeAnnotation, bool)>) {
        self.interfaces.insert(name, members);
    }

    /// 获取变量类型
    fn get_variable(&self, name: &str) -> Option<&TypeAnnotation> {
        self.variables.get(name)
    }

    /// 获取函数类型
    fn get_function(&self, name: &str) -> Option<&(Vec<TypeAnnotation>, Option<TypeAnnotation>)> {
        self.functions.get(name)
    }

    /// 获取类型别名
    fn get_type_alias(&self, name: &str) -> Option<&TypeAnnotation> {
        self.type_aliases.get(name)
    }

    /// 获取接口
    fn get_interface(&self, name: &str) -> Option<&Vec<(String, TypeAnnotation, bool)>> {
        self.interfaces.get(name)
    }

    /// 检查类型是否兼容
    fn is_compatible(&self, from: &TypeAnnotation, to: &TypeAnnotation) -> bool {
        is_compatible(self, from, to)
    }

    /// 解析类型引用
    fn resolve_type(&self, ty: &TypeAnnotation) -> TypeAnnotation {
        match ty {
            TypeAnnotation::TypeReference(name) => {
                if let Some(alias) = self.get_type_alias(name) {
                    self.resolve_type(alias)
                }
                else {
                    ty.clone()
                }
            }
            _ => ty.clone(),
        }
    }
}

/// 检查两个类型是否兼容
fn is_compatible(env: &TypeEnvironment, from: &TypeAnnotation, to: &TypeAnnotation) -> bool {
    let from = env.resolve_type(from);
    let to = env.resolve_type(to);

    match (&from, &to) {
        // 任何类型都兼容 Any
        (_, TypeAnnotation::Any) => true,
        // Any 只兼容 Any
        (TypeAnnotation::Any, _) => false,
        // 基本类型兼容：相同类型或 null/undefined 相互兼容
        (TypeAnnotation::Primitive(a), TypeAnnotation::Primitive(b)) => {
            a == b
                || (*a == PrimitiveType::Null && *b == PrimitiveType::Undefined)
                || (*a == PrimitiveType::Undefined && *b == PrimitiveType::Null)
        }
        // 数组类型兼容：元素类型兼容
        (TypeAnnotation::Array(a), TypeAnnotation::Array(b)) => is_compatible(env, a, b),
        // 对象类型兼容：to 中的所有成员在 from 中都存在且类型兼容
        (TypeAnnotation::Object(from_members), TypeAnnotation::Object(to_members)) => {
            let from_map: std::collections::HashMap<_, _> = from_members.iter().map(|(name, ty)| (name, ty)).collect();

            to_members
                .iter()
                .all(|(name, ty)| from_map.get(name).map(|from_ty| is_compatible(env, from_ty, ty)).unwrap_or(false))
        }
        // 联合类型兼容：from 是 to 的子类型
        (_, TypeAnnotation::Union(to_types)) => to_types.iter().any(|to_ty| is_compatible(env, &from, to_ty)),
        // 函数类型兼容：参数类型逆变，返回类型协变
        (
            TypeAnnotation::Function { params: from_params, return_type: from_return },
            TypeAnnotation::Function { params: to_params, return_type: to_return },
        ) => {
            if from_params.len() != to_params.len() {
                return false;
            }

            // 参数类型逆变：from 参数类型要比 to 参数类型更宽泛
            let params_compatible =
                from_params.iter().zip(to_params.iter()).all(|(from_param, to_param)| is_compatible(env, to_param, from_param));

            // 返回类型协变：from 返回类型要比 to 返回类型更具体
            let return_compatible = is_compatible(env, from_return, to_return);

            params_compatible && return_compatible
        }
        // 泛型类型兼容：需要更复杂的处理，这里简化处理
        (TypeAnnotation::Generic(name1, args1), TypeAnnotation::Generic(name2, args2)) => {
            name1 == name2
                && args1.len() == args2.len()
                && args1.iter().zip(args2.iter()).all(|(a, b)| is_compatible(env, a, b))
        }
        // 元组类型兼容：每个位置的类型都兼容
        (TypeAnnotation::Tuple(from_types), TypeAnnotation::Tuple(to_types)) => {
            from_types.len() == to_types.len() && from_types.iter().zip(to_types.iter()).all(|(a, b)| is_compatible(env, a, b))
        }
        // 其他情况不兼容
        _ => false,
    }
}

/// 类型检查函数
pub fn check(program: Program) -> Result<Program, TsError> {
    let mut env = TypeEnvironment::new();

    // 添加全局变量到类型环境
    env.add_variable("console".to_string(), TypeAnnotation::Any);

    // 检查所有语句
    for statement in &program.statements {
        check_statement(statement, &mut env)?;
    }

    Ok(program)
}

/// 检查语句
fn check_statement(statement: &Statement, env: &mut TypeEnvironment) -> Result<(), TsError> {
    match statement {
        Statement::VariableDeclaration { name, ty, initializer } => {
            // 检查初始化表达式的类型
            if let Some(init) = initializer {
                let init_type = check_expression(init, env)?;
                // 如果有显式类型注解，检查类型兼容性
                if let Some(ty) = ty {
                    if !env.is_compatible(&init_type, ty) {
                        return Err(TsError::TypeError(format!(
                            "Type '{}' is not assignable to type '{}'",
                            format_type(&init_type),
                            format_type(ty)
                        )));
                    }
                    // 添加变量到类型环境
                    env.add_variable(name.clone(), ty.clone());
                }
                else {
                    // 没有显式类型注解，使用初始化表达式的类型
                    env.add_variable(name.clone(), init_type);
                }
            }
            else if let Some(ty) = ty {
                // 只有类型注解，没有初始化表达式
                env.add_variable(name.clone(), ty.clone());
            }
            else {
                // 既没有类型注解也没有初始化表达式，默认为 any 类型
                env.add_variable(name.clone(), TypeAnnotation::Any);
            }
        }
        Statement::FunctionDeclaration { name, params, return_type, body } => {
            // 检查函数体
            let mut func_env = env.clone();
            // 添加参数到函数环境
            for (i, param) in params.iter().enumerate() {
                // 暂时使用 any 类型，后续可以根据函数签名进行类型推断
                func_env.add_variable(param.clone(), TypeAnnotation::Any);
            }
            // 检查函数体语句
            for stmt in body {
                check_statement(stmt, &mut func_env)?;
            }
            // 添加函数到类型环境
            env.add_function(name.clone(), vec![TypeAnnotation::Any; params.len()], return_type.clone());
        }
        Statement::ClassDeclaration { name, super_class, methods } => {
            // 检查方法
            for method in methods {
                let mut method_env = env.clone();
                // 添加方法参数到环境
                for param in &method.params {
                    method_env.add_variable(param.clone(), TypeAnnotation::Any);
                }
                // 检查方法体
                for stmt in &method.body {
                    check_statement(stmt, &mut method_env)?;
                }
            }
        }
        Statement::InterfaceDeclaration { name, extends, members } => {
            let mut interface_members = vec![];
            // 处理继承的接口
            for base in extends {
                if let Some(base_members) = env.get_interface(base) {
                    interface_members.extend(base_members.clone());
                }
            }
            // 添加接口成员
            for member in members {
                match member {
                    InterfaceMember::Property { name, ty, optional } => {
                        interface_members.push((name.clone(), ty.clone(), *optional));
                    }
                    InterfaceMember::Method { name, params, return_type, optional } => {
                        // 暂时忽略方法参数类型，后续可以完善
                        interface_members.push((name.clone(), return_type.clone(), *optional));
                    }
                }
            }
            // 添加接口到类型环境
            env.add_interface(name.clone(), interface_members);
        }
        Statement::TypeAlias { name, ty } => {
            // 添加类型别名到类型环境
            env.add_type_alias(name.clone(), ty.clone());
        }
        Statement::If { test, consequent, alternate } => {
            // 检查条件表达式
            check_expression(test, env)?;
            // 检查 consequent 语句
            check_statement(consequent, env)?;
            // 检查 alternate 语句
            if let Some(alt) = alternate {
                check_statement(alt, env)?;
            }
        }
        Statement::While { test, body } => {
            // 检查条件表达式
            check_expression(test, env)?;
            // 检查 body 语句
            check_statement(body, env)?;
        }
        Statement::For { init, test, update, body } => {
            // 检查 init 语句
            if let Some(init_stmt) = init {
                check_statement(init_stmt, env)?;
            }
            // 检查 test 表达式
            if let Some(test_expr) = test {
                check_expression(test_expr, env)?;
            }
            // 检查 update 表达式
            if let Some(update_expr) = update {
                check_expression(update_expr, env)?;
            }
            // 检查 body 语句
            check_statement(body, env)?;
        }
        Statement::Return(expr) => {
            // 检查返回表达式
            if let Some(expr) = expr {
                check_expression(expr, env)?;
            }
        }
        Statement::Block(statements) => {
            // 检查块内所有语句
            for stmt in statements {
                check_statement(stmt, env)?;
            }
        }
        Statement::Expression(expr) => {
            // 检查表达式
            check_expression(expr, env)?;
        }
        _ => {
            // 其他语句不需要类型检查
        }
    }

    Ok(())
}

/// 检查表达式并返回其类型
fn check_expression(expr: &Expression, env: &mut TypeEnvironment) -> Result<TypeAnnotation, TsError> {
    match expr {
        Expression::Literal(value) => {
            // 根据字面量值推断类型
            let ty = match value {
                typescript_types::TsValue::Undefined => TypeAnnotation::Primitive(PrimitiveType::Undefined),
                typescript_types::TsValue::Null => TypeAnnotation::Primitive(PrimitiveType::Null),
                typescript_types::TsValue::Boolean(_) => TypeAnnotation::Primitive(PrimitiveType::Boolean),
                typescript_types::TsValue::Number(_) => TypeAnnotation::Primitive(PrimitiveType::Number),
                typescript_types::TsValue::String(_) => TypeAnnotation::Primitive(PrimitiveType::String),
                typescript_types::TsValue::Object(_) => TypeAnnotation::Object(vec![]),
                typescript_types::TsValue::Array(_) => TypeAnnotation::Array(Box::new(TypeAnnotation::Any)),
                typescript_types::TsValue::Function(_) => {
                    TypeAnnotation::Function { params: vec![], return_type: Box::new(TypeAnnotation::Any) }
                }
                typescript_types::TsValue::Error(_) => TypeAnnotation::Any,
                typescript_types::TsValue::Union(_) => TypeAnnotation::Any,
                typescript_types::TsValue::Generic(_, _) => TypeAnnotation::Any,
                typescript_types::TsValue::Symbol(_) => TypeAnnotation::Primitive(PrimitiveType::Symbol),
                typescript_types::TsValue::BigInt(_) => TypeAnnotation::Primitive(PrimitiveType::BigInt),
                typescript_types::TsValue::Date(_) => TypeAnnotation::Any,
                typescript_types::TsValue::RegExp(_) => TypeAnnotation::Any,
                typescript_types::TsValue::Map(_) => TypeAnnotation::Any,
                typescript_types::TsValue::Set(_) => TypeAnnotation::Any,
                typescript_types::TsValue::Promise(_) => TypeAnnotation::Any,
                typescript_types::TsValue::Iterable(_) => TypeAnnotation::Any,
            };
            Ok(ty)
        }
        Expression::Identifier(name) => {
            // 查找变量类型
            if let Some(ty) = env.get_variable(name) {
                Ok(ty.clone())
            }
            else if let Some(ty) = env.get_type_alias(name) {
                Ok(ty.clone())
            }
            else {
                Err(TsError::ReferenceError(format!("Variable '{}' is not defined", name)))
            }
        }
        Expression::Binary { left, op, right } => {
            // 检查左右表达式
            let left_type = check_expression(left, env)?;
            let right_type = check_expression(right, env)?;
            // 简化的类型检查，实际需要根据操作符进行更复杂的类型检查
            Ok(TypeAnnotation::Any)
        }
        Expression::Unary { op, expr } => {
            // 检查表达式
            let expr_type = check_expression(expr, env)?;
            // 简化的类型检查
            Ok(TypeAnnotation::Any)
        }
        Expression::Call { callee, args } => {
            // 检查调用表达式
            let callee_type = check_expression(callee, env)?;
            // 检查参数
            let arg_types: Vec<TypeAnnotation> = args.iter().map(|arg| check_expression(arg, env)).collect::<Result<_, _>>()?;

            // 根据函数类型检查参数类型
            match callee_type {
                TypeAnnotation::Function { params: func_params, return_type } => {
                    if arg_types.len() != func_params.len() {
                        return Err(TsError::TypeError(format!(
                            "Expected {} arguments, but got {}",
                            func_params.len(),
                            arg_types.len()
                        )));
                    }

                    // 检查每个参数的类型是否兼容
                    for (i, (arg_type, func_param_type)) in arg_types.iter().zip(func_params.iter()).enumerate() {
                        if !env.is_compatible(arg_type, func_param_type) {
                            return Err(TsError::TypeError(format!(
                                "Argument {} type '{}' is not compatible with parameter type '{}'",
                                i,
                                format_type(arg_type),
                                format_type(func_param_type)
                            )));
                        }
                    }

                    Ok(*return_type)
                }
                _ => {
                    // 非函数类型调用，返回 Any 类型
                    Ok(TypeAnnotation::Any)
                }
            }
        }
        Expression::Member { object, property } => {
            // 检查对象表达式
            let object_type = check_expression(object, env)?;
            // 检查属性表达式，但如果是 Identifier，不需要检查它是否是变量
            match property.as_ref() {
                Expression::Identifier(_) => {
                    // 对于标识符属性，不需要检查是否是变量
                }
                _ => {
                    // 对于其他类型的属性表达式，需要检查
                    check_expression(property, env)?;
                }
            }
            // 简化的类型检查
            Ok(TypeAnnotation::Any)
        }
        Expression::Index { object, index } => {
            // 检查对象表达式
            let object_type = check_expression(object, env)?;
            // 检查索引表达式
            check_expression(index, env)?;
            // 简化的类型检查
            Ok(TypeAnnotation::Any)
        }
        Expression::Object(properties) => {
            // 检查属性值
            let mut object_type = vec![];
            for (name, value) in properties {
                let value_type = check_expression(value, env)?;
                object_type.push((name.clone(), value_type));
            }
            Ok(TypeAnnotation::Object(object_type))
        }
        Expression::Array(elements) => {
            // 检查元素并推断数组元素类型
            if elements.is_empty() {
                // 空数组默认为 any[]
                Ok(TypeAnnotation::Array(Box::new(TypeAnnotation::Any)))
            }
            else {
                // 检查所有元素并推断共同类型
                let mut element_types: Vec<TypeAnnotation> = vec![];
                for element in elements {
                    let elem_type = check_expression(element, env)?;
                    element_types.push(elem_type);
                }

                // 推断共同类型
                let common_type = infer_common_type(env, &element_types);
                Ok(TypeAnnotation::Array(Box::new(common_type)))
            }
        }
        Expression::Assignment { left, op, right } => {
            // 检查左右表达式
            let left_type = check_expression(left, env)?;
            let right_type = check_expression(right, env)?;
            // 检查类型兼容性
            if !env.is_compatible(&right_type, &left_type) {
                return Err(TsError::TypeError(format!(
                    "Type '{}' is not assignable to type '{}'",
                    format_type(&right_type),
                    format_type(&left_type)
                )));
            }
            Ok(left_type)
        }
        Expression::Conditional { test, consequent, alternate } => {
            // 检查条件表达式
            check_expression(test, env)?;
            // 检查 consequent 表达式
            let consequent_type = check_expression(consequent, env)?;
            // 检查 alternate 表达式
            let alternate_type = check_expression(alternate, env)?;
            // 计算联合类型
            Ok(TypeAnnotation::Union(vec![consequent_type, alternate_type]))
        }
        Expression::Function { params, body } => {
            // 检查函数体
            let mut func_env = env.clone();
            // 添加参数到函数环境
            for param in params {
                func_env.add_variable(param.clone(), TypeAnnotation::Any);
            }
            // 检查函数体语句
            for stmt in body {
                check_statement(stmt, &mut func_env)?;
            }
            // 返回函数类型
            Ok(TypeAnnotation::Function {
                params: vec![TypeAnnotation::Any; params.len()],
                return_type: Box::new(TypeAnnotation::Any),
            })
        }
        Expression::ArrowFunction { params, body } => {
            // 检查函数体
            let mut func_env = env.clone();
            // 添加参数到函数环境
            for param in params {
                func_env.add_variable(param.clone(), TypeAnnotation::Any);
            }
            // 检查函数体表达式
            let return_type = check_expression(body, &mut func_env)?;
            // 返回函数类型
            Ok(TypeAnnotation::Function { params: vec![TypeAnnotation::Any; params.len()], return_type: Box::new(return_type) })
        }
    }
}

/// 接口成员（从 typescript-ir 导入）
use typescript_ir::InterfaceMember;

/// 将类型注解转换为字符串
fn format_type(ty: &TypeAnnotation) -> String {
    match ty {
        TypeAnnotation::Primitive(prim) => format!("{:?}", prim),
        TypeAnnotation::Array(elem) => format!("{}[]", format_type(elem)),
        TypeAnnotation::Object(members) => {
            let members_str: Vec<String> = members.iter().map(|(name, ty)| format!("{}: {}", name, format_type(ty))).collect();
            format!("{{ {} }}", members_str.join(", "))
        }
        TypeAnnotation::Union(types) => {
            let types_str: Vec<String> = types.iter().map(format_type).collect();
            format!("({})", types_str.join(" | "))
        }
        TypeAnnotation::Intersection(types) => {
            let types_str: Vec<String> = types.iter().map(format_type).collect();
            format!("({})", types_str.join(" & "))
        }
        TypeAnnotation::Generic(name, args) => {
            let args_str: Vec<String> = args.iter().map(format_type).collect();
            format!("{}<{}>", name, args_str.join(", "))
        }
        TypeAnnotation::Function { params, return_type } => {
            let params_str: Vec<String> = params.iter().map(format_type).collect();
            format!("({}) => {}", params_str.join(", "), format_type(return_type))
        }
        TypeAnnotation::TypeReference(name) => name.clone(),
        TypeAnnotation::Any => "any".to_string(),
        TypeAnnotation::Unknown => "unknown".to_string(),
        TypeAnnotation::Void => "void".to_string(),
        TypeAnnotation::Never => "never".to_string(),
        TypeAnnotation::Tuple(types) => {
            let types_str: Vec<String> = types.iter().map(format_type).collect();
            format!("[{}]", types_str.join(", "))
        }
    }
}

/// 推断多个类型的共同类型
fn infer_common_type(env: &TypeEnvironment, types: &[TypeAnnotation]) -> TypeAnnotation {
    if types.is_empty() {
        return TypeAnnotation::Any;
    }

    // 初始化为第一个类型
    let mut common_type = types[0].clone();

    // 遍历所有类型，找到共同类型
    for ty in types.iter().skip(1) {
        common_type = find_common_type(env, &common_type, ty);
    }

    common_type
}

/// 找到两个类型的共同类型
fn find_common_type(env: &TypeEnvironment, a: &TypeAnnotation, b: &TypeAnnotation) -> TypeAnnotation {
    // 如果其中一个类型兼容另一个类型，返回更宽泛的类型
    if env.is_compatible(a, b) {
        return b.clone();
    }
    if env.is_compatible(b, a) {
        return a.clone();
    }

    // 否则返回联合类型
    TypeAnnotation::Union(vec![a.clone(), b.clone()])
}
