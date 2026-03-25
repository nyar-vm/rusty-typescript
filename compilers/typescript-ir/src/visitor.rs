//! TypeScript IR 访问者模块

use crate::{
    expression::Expression,
    program::{Module, Program},
    statement::Statement,
    types::TypeAnnotation,
};

/// IR 访问者 trait
pub trait Visitor<T> {
    /// 访问表达式
    fn visit_expression(&mut self, expr: &Expression) -> T;
    /// 访问语句
    fn visit_statement(&mut self, stmt: &Statement) -> T;
    /// 访问类型注解
    fn visit_type_annotation(&mut self, ty: &TypeAnnotation) -> T;
    /// 访问程序
    fn visit_program(&mut self, program: &Program) -> T;
    /// 访问模块
    fn visit_module(&mut self, module: &Module) -> T;
}

/// 简单的 IR 遍历实现
pub struct SimpleVisitor {
    /// 访问结果
    pub result: Vec<String>,
}

impl SimpleVisitor {
    /// 创建一个新的简单访问者
    pub fn new() -> Self {
        Self { result: Vec::new() }
    }
}

impl Default for SimpleVisitor {
    fn default() -> Self {
        Self::new()
    }
}

impl Visitor<()> for SimpleVisitor {
    fn visit_expression(&mut self, expr: &Expression) -> () {
        match expr {
            Expression::Literal(_) => self.result.push("Literal".to_string()),
            Expression::Identifier(name) => self.result.push(format!("Identifier: {}", name)),
            Expression::Binary { left, op: _, right } => {
                self.result.push("Binary".to_string());
                self.visit_expression(left);
                self.visit_expression(right);
            }
            Expression::Unary { op: _, expr } => {
                self.result.push("Unary".to_string());
                self.visit_expression(expr);
            }
            Expression::Call { callee, args } => {
                self.result.push("Call".to_string());
                self.visit_expression(callee);
                for arg in args {
                    self.visit_expression(arg);
                }
            }
            Expression::Member { object, property } => {
                self.result.push("Member".to_string());
                self.visit_expression(object);
                self.visit_expression(property);
            }
            Expression::Index { object, index } => {
                self.result.push("Index".to_string());
                self.visit_expression(object);
                self.visit_expression(index);
            }
            Expression::Object(properties) => {
                self.result.push("Object".to_string());
                for (_, value) in properties {
                    self.visit_expression(value);
                }
            }
            Expression::Array(elements) => {
                self.result.push("Array".to_string());
                for elem in elements {
                    self.visit_expression(elem);
                }
            }
            Expression::Assignment { left, op: _, right } => {
                self.result.push("Assignment".to_string());
                self.visit_expression(left);
                self.visit_expression(right);
            }
            Expression::Conditional { test, consequent, alternate } => {
                self.result.push("Conditional".to_string());
                self.visit_expression(test);
                self.visit_expression(consequent);
                self.visit_expression(alternate);
            }
            Expression::Function { params: _, body } => {
                self.result.push("Function".to_string());
                for stmt in body {
                    self.visit_statement(stmt);
                }
            }
            Expression::ArrowFunction { params: _, body } => {
                self.result.push("ArrowFunction".to_string());
                self.visit_expression(body);
            }
        }
    }

    fn visit_statement(&mut self, stmt: &Statement) -> () {
        match stmt {
            Statement::Expression(expr) => {
                self.result.push("ExpressionStatement".to_string());
                self.visit_expression(expr);
            }
            Statement::VariableDeclaration { name, ty: _, initializer } => {
                self.result.push(format!("VariableDeclaration: {}", name));
                if let Some(init) = initializer {
                    self.visit_expression(init);
                }
            }
            Statement::Block(statements) => {
                self.result.push("Block".to_string());
                for stmt in statements {
                    self.visit_statement(stmt);
                }
            }
            Statement::If { test, consequent, alternate } => {
                self.result.push("If".to_string());
                self.visit_expression(test);
                self.visit_statement(consequent);
                if let Some(alt) = alternate {
                    self.visit_statement(alt);
                }
            }
            Statement::While { test, body } => {
                self.result.push("While".to_string());
                self.visit_expression(test);
                self.visit_statement(body);
            }
            Statement::For { init, test, update, body } => {
                self.result.push("For".to_string());
                if let Some(init_stmt) = init {
                    self.visit_statement(init_stmt);
                }
                if let Some(test_expr) = test {
                    self.visit_expression(test_expr);
                }
                if let Some(update_expr) = update {
                    self.visit_expression(update_expr);
                }
                self.visit_statement(body);
            }
            Statement::Return(expr) => {
                self.result.push("Return".to_string());
                if let Some(expr) = expr {
                    self.visit_expression(expr);
                }
            }
            Statement::Break => self.result.push("Break".to_string()),
            Statement::Continue => self.result.push("Continue".to_string()),
            Statement::FunctionDeclaration { name, params: _, return_type: _, body, .. } => {
                self.result.push(format!("FunctionDeclaration: {}", name));
                for stmt in body {
                    self.visit_statement(stmt);
                }
            }
            Statement::ClassDeclaration { name, super_class: _, methods, .. } => {
                self.result.push(format!("ClassDeclaration: {}", name));
                for method in methods {
                    for stmt in &method.body {
                        self.visit_statement(stmt);
                    }
                }
            }
            Statement::InterfaceDeclaration { name, .. } => {
                self.result.push(format!("InterfaceDeclaration: {}", name));
            }
            Statement::TypeAlias { name, ty, .. } => {
                self.result.push(format!("TypeAlias: {}", name));
                self.visit_type_annotation(ty);
            }
            Statement::EnumDeclaration { name, members, is_const, is_declare: _ } => {
                self.result.push(format!("EnumDeclaration: {}{}", if *is_const { "const " } else { "" }, name));
                for member in members {
                    if let Some(value) = &member.value {
                        self.visit_expression(value);
                    }
                }
            }
            Statement::NamespaceDeclaration { name, body, is_exported } => {
                self.result.push(format!("NamespaceDeclaration: {}{}", if *is_exported { "export " } else { "" }, name));
                for stmt in body {
                    self.visit_statement(stmt);
                }
            }
            Statement::ModuleDeclaration { name, body } => {
                self.result.push(format!("ModuleDeclaration: {}", name));
                for stmt in body {
                    self.visit_statement(stmt);
                }
            }
            Statement::ImportDeclaration { source, specifiers: _, is_type_only } => {
                self.result.push(format!("ImportDeclaration: {}{}", if *is_type_only { "type " } else { "" }, source));
            }
            Statement::ExportDeclaration { declaration: _, is_default } => {
                self.result.push(format!("ExportDeclaration: {}", if *is_default { "default" } else { "" }));
            }
        }
    }

    fn visit_type_annotation(&mut self, ty: &TypeAnnotation) -> () {
        match ty {
            TypeAnnotation::Primitive(_) => self.result.push("PrimitiveType".to_string()),
            TypeAnnotation::Array(elem_ty) => {
                self.result.push("ArrayType".to_string());
                self.visit_type_annotation(elem_ty);
            }
            TypeAnnotation::Object(properties) => {
                self.result.push("ObjectType".to_string());
                for (_, ty) in properties {
                    self.visit_type_annotation(ty);
                }
            }
            TypeAnnotation::Union(types) => {
                self.result.push("UnionType".to_string());
                for ty in types {
                    self.visit_type_annotation(ty);
                }
            }
            TypeAnnotation::Intersection(types) => {
                self.result.push("IntersectionType".to_string());
                for ty in types {
                    self.visit_type_annotation(ty);
                }
            }
            TypeAnnotation::Generic { name, args } => {
                self.result.push(format!("GenericType: {}", name));
                for arg in args {
                    self.visit_type_annotation(arg);
                }
            }
            TypeAnnotation::Function { params, return_type } => {
                self.result.push("FunctionType".to_string());
                for param in params {
                    self.visit_type_annotation(param);
                }
                self.visit_type_annotation(return_type);
            }
            TypeAnnotation::TypeReference(name) => {
                self.result.push(format!("TypeReference: {}", name));
            }
            TypeAnnotation::Any => self.result.push("AnyType".to_string()),
            TypeAnnotation::Unknown => self.result.push("UnknownType".to_string()),
            TypeAnnotation::Void => self.result.push("VoidType".to_string()),
            TypeAnnotation::Never => self.result.push("NeverType".to_string()),
            TypeAnnotation::Tuple(types) => {
                self.result.push("TupleType".to_string());
                for ty in types {
                    self.visit_type_annotation(ty);
                }
            }
        }
    }

    fn visit_program(&mut self, program: &Program) -> () {
        self.result.push("Program".to_string());
        for stmt in &program.statements {
            self.visit_statement(stmt);
        }
    }

    fn visit_module(&mut self, module: &Module) -> () {
        self.result.push(format!("Module: {}", module.name));
        for stmt in &module.statements {
            self.visit_statement(stmt);
        }
    }
}
