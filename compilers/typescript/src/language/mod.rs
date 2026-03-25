#![warn(missing_docs)]

use oak_typescript::language::TypeScriptLanguage as OakTypeScriptLanguage;

/// TypeScript 语言配置
///
/// 扩展了 oak-typescript 的语言配置，支持更多 TypeScript 语法特性
pub type TypeScriptLanguage = OakTypeScriptLanguage;

/// TypeScript 语法特性
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeScriptFeature {
    /// 类型注解
    TypeAnnotations,
    /// 接口定义
    Interfaces,
    /// 泛型
    Generics,
    /// 枚举
    Enums,
    /// 命名空间
    Namespaces,
    /// 装饰器
    Decorators,
    /// 异步/等待
    AsyncAwait,
    /// 可选链
    OptionalChaining,
    /// 空值合并
    NullishCoalescing,
    /// 模板字面量类型
    TemplateLiteralTypes,
    /// 映射类型
    MappedTypes,
    /// 条件类型
    ConditionalTypes,
    /// 类型别名
    TypeAliases,
    /// 联合类型
    UnionTypes,
    /// 交叉类型
    IntersectionTypes,
    /// 字面量类型
    LiteralTypes,
    /// 元组类型
    TupleTypes,
    /// 函数类型
    FunctionTypes,
    /// 对象类型
    ObjectTypes,
    /// 数组类型
    ArrayTypes,
    /// 索引访问类型
    IndexedAccessTypes,
    /// 键值类型
    KeyofTypes,
    /// typeof 类型
    TypeofTypes,
    /// 推断类型
    InferredTypes,
    /// 递归类型
    RecursiveTypes,
    /// 泛型约束
    GenericConstraints,
    /// 类型参数默认值
    TypeParameterDefaults,
    /// 剩余类型参数
    RestTypeParameters,
    /// 可选类型参数
    OptionalTypeParameters,
    /// 重载函数类型
    OverloadedFunctionTypes,
    /// 构造函数类型
    ConstructorTypes,
    /// 访问修饰符
    AccessModifiers,
    /// 静态成员
    StaticMembers,
    /// 抽象类和方法
    AbstractClasses,
    /// 继承
    Inheritance,
    /// 实现接口
    InterfaceImplementation,
    /// 模块系统
    ModuleSystem,
    /// ES 模块
    EsModules,
    /// CommonJS 模块
    CommonJsModules,
    /// 外部模块声明
    ExternalModuleDeclarations,
    /// 三斜杠指令
    TripleSlashDirectives,
    /// JSDoc 注释
    JsDocComments,
    /// 严格模式
    StrictMode,
    /// 严格空检查
    StrictNullChecks,
    /// 严格函数类型
    StrictFunctionTypes,
    /// 严格绑定调用
    StrictBindCallApply,
    /// 严格属性初始化
    StrictPropertyInitialization,
    /// 无隐式 any
    NoImplicitAny,
    /// 无隐式 this
    NoImplicitThis,
    /// 总是严格模式
    AlwaysStrict,
    /// 禁止未使用的变量
    NoUnusedVariables,
    /// 禁止未使用的参数
    NoUnusedParameters,
    /// 禁止空函数
    NoEmptyFunctions,
    /// 禁止空接口
    NoEmptyInterfaces,
    /// 禁止重复的导入
    NoDuplicateImports,
    /// 禁止重复的类成员
    NoDuplicateClassMembers,
    /// 禁止重复的函数参数
    NoDuplicateParameters,
    /// 禁止重复的 case 标签
    NoDuplicateCaseLabels,
    /// 禁止不可达代码
    NoUnreachableCode,
    /// 禁止未返回值的函数
    NoImplicitReturns,
    /// 禁止赋值表达式中的类型转换
    NoImplicitCoercion,
    /// 禁止使用 eval
    NoEval,
    /// 禁止使用 with
    NoWith,
    /// 禁止使用 arguments
    NoArguments,
    /// 禁止使用 delete
    NoDelete,
    /// 禁止使用 void
    NoVoid,
    /// 禁止使用 undefined
    NoUndefined,
    /// 禁止使用 null
    NoNull,
    /// 禁止使用 any
    NoAny,
    /// 禁止使用 never
    NoNever,
    /// 禁止使用 object
    NoObject,
    /// 禁止使用 unknown
    NoUnknown,
    /// 禁止使用 symbol
    NoSymbol,
    /// 禁止使用 bigint
    NoBigInt,
    /// 禁止使用 number
    NoNumber,
    /// 禁止使用 string
    NoString,
    /// 禁止使用 boolean
    NoBoolean,
    /// 禁止使用 array
    NoArray,
    /// 禁止使用 tuple
    NoTuple,
    /// 禁止使用 enum
    NoEnum,
    /// 禁止使用 interface
    NoInterface,
    /// 禁止使用 type
    NoType,
    /// 禁止使用 class
    NoClass,
    /// 禁止使用 function
    NoFunction,
    /// 禁止使用 const
    NoConst,
    /// 禁止使用 let
    NoLet,
    /// 禁止使用 var
    NoVar,
    /// 禁止使用 async
    NoAsync,
    /// 禁止使用 await
    NoAwait,
    /// 禁止使用 yield
    NoYield,
    /// 禁止使用 return
    NoReturn,
    /// 禁止使用 throw
    NoThrow,
    /// 禁止使用 try
    NoTry,
    /// 禁止使用 catch
    NoCatch,
    /// 禁止使用 finally
    NoFinally,
    /// 禁止使用 if
    NoIf,
    /// 禁止使用 else
    NoElse,
    /// 禁止使用 switch
    NoSwitch,
    /// 禁止使用 case
    NoCase,
    /// 禁止使用 default
    NoDefault,
    /// 禁止使用 for
    NoFor,
    /// 禁止使用 while
    NoWhile,
    /// 禁止使用 do
    NoDo,
    /// 禁止使用 break
    NoBreak,
    /// 禁止使用 continue
    NoContinue,
    /// 禁止使用 with
    NoWithStatement,
    /// 禁止使用 debugger
    NoDebugger,
    /// 禁止使用 import
    NoImport,
    /// 禁止使用 export
    NoExport,
    /// 禁止使用 from
    NoFrom,
    /// 禁止使用 as
    NoAs,
    /// 禁止使用 in
    NoIn,
    /// 禁止使用 instanceof
    NoInstanceOf,
    /// 禁止使用 new
    NoNew,
    /// 禁止使用 this
    NoThis,
    /// 禁止使用 super
    NoSuper,
    /// 禁止使用 static
    NoStatic,
    /// 禁止使用 public
    NoPublic,
    /// 禁止使用 private
    NoPrivate,
    /// 禁止使用 protected
    NoProtected,
    /// 禁止使用 abstract
    NoAbstract,
    /// 禁止使用 readonly
    NoReadonly,
    /// 禁止使用 optional
    NoOptional,
    /// 禁止使用 rest
    NoRest,
    /// 禁止使用 spread
    NoSpread,
    /// 禁止使用 destructuring
    NoDestructuring,
    /// 禁止使用 arrow functions
    NoArrowFunctions,
    /// 禁止使用 generator functions
    NoGeneratorFunctions,
    /// 禁止使用 async functions
    NoAsyncFunctions,
    /// 禁止使用 method syntax
    NoMethodSyntax,
    /// 禁止使用 computed properties
    NoComputedProperties,
    /// 禁止使用 shorthand properties
    NoShorthandProperties,
    /// 禁止使用 rest properties
    NoRestProperties,
    /// 禁止使用 spread properties
    NoSpreadProperties,
    /// 禁止使用 template literals
    NoTemplateLiterals,
    /// 禁止使用 regex literals
    NoRegexLiterals,
    /// 禁止使用 numeric literals
    NoNumericLiterals,
    /// 禁止使用 string literals
    NoStringLiterals,
    /// 禁止使用 boolean literals
    NoBooleanLiterals,
    /// 禁止使用 null literals
    NoNullLiterals,
    /// 禁止使用 undefined literals
    NoUndefinedLiterals,
    /// 禁止使用 bigint literals
    NoBigIntLiterals,
    /// 禁止使用 symbol literals
    NoSymbolLiterals,
    /// 禁止使用 object literals
    NoObjectLiterals,
    /// 禁止使用 array literals
    NoArrayLiterals,
    /// 禁止使用 function literals
    NoFunctionLiterals,
    /// 禁止使用 class literals
    NoClassLiterals,
    /// 禁止使用 namespace literals
    NoNamespaceLiterals,
    /// 禁止使用 module literals
    NoModuleLiterals,
    /// 禁止使用 enum literals
    NoEnumLiterals,
    /// 禁止使用 type literals
    NoTypeLiterals,
    /// 禁止使用 interface literals
    NoInterfaceLiterals,
    /// 禁止使用 union literals
    NoUnionLiterals,
    /// 禁止使用 intersection literals
    NoIntersectionLiterals,
    /// 禁止使用 literal types
    NoLiteralTypes,
    /// 禁止使用 tuple types
    NoTupleTypes,
    /// 禁止使用 function types
    NoFunctionTypes,
    /// 禁止使用 object types
    NoObjectTypes,
    /// 禁止使用 array types
    NoArrayTypes,
    /// 禁止使用 indexed access types
    NoIndexedAccessTypes,
    /// 禁止使用 keyof types
    NoKeyofTypes,
    /// 禁止使用 typeof types
    NoTypeofTypes,
    /// 禁止使用 inferred types
    NoInferredTypes,
    /// 禁止使用 recursive types
    NoRecursiveTypes,
    /// 禁止使用 generic constraints
    NoGenericConstraints,
    /// 禁止使用 type parameter defaults
    NoTypeParameterDefaults,
    /// 禁止使用 rest type parameters
    NoRestTypeParameters,
    /// 禁止使用 optional type parameters
    NoOptionalTypeParameters,
    /// 禁止使用 overloaded function types
    NoOverloadedFunctionTypes,
    /// 禁止使用 constructor types
    NoConstructorTypes,
    /// 禁止使用 access modifiers
    NoAccessModifiers,
    /// 禁止使用 static members
    NoStaticMembers,
    /// 禁止使用 abstract classes and methods
    NoAbstractClasses,
    /// 禁止使用 inheritance
    NoInheritance,
    /// 禁止使用 interface implementation
    NoInterfaceImplementation,
    /// 禁止使用 module system
    NoModuleSystem,
    /// 禁止使用 ES modules
    NoEsModules,
    /// 禁止使用 CommonJS modules
    NoCommonJsModules,
    /// 禁止使用 external module declarations
    NoExternalModuleDeclarations,
    /// 禁止使用 triple-slash directives
    NoTripleSlashDirectives,
    /// 禁止使用 JSDoc comments
    NoJsDocComments,
    /// 禁止使用 strict mode
    NoStrictMode,
    /// 禁止使用 strict null checks
    NoStrictNullChecks,
    /// 禁止使用 strict function types
    NoStrictFunctionTypes,
    /// 禁止使用 strict bind call apply
    NoStrictBindCallApply,
    /// 禁止使用 strict property initialization
    NoStrictPropertyInitialization,
    /// 禁止使用 no implicit any
    NoNoImplicitAny,
    /// 禁止使用 no implicit this
    NoNoImplicitThis,
    /// 禁止使用 always strict
    NoAlwaysStrict,
    /// 禁止使用 no unused variables
    NoNoUnusedVariables,
    /// 禁止使用 no unused parameters
    NoNoUnusedParameters,
    /// 禁止使用 no empty functions
    NoNoEmptyFunctions,
    /// 禁止使用 no empty interfaces
    NoNoEmptyInterfaces,
    /// 禁止使用 no duplicate imports
    NoNoDuplicateImports,
    /// 禁止使用 no duplicate class members
    NoNoDuplicateClassMembers,
    /// 禁止使用 no duplicate parameters
    NoNoDuplicateParameters,
    /// 禁止使用 no duplicate case labels
    NoNoDuplicateCaseLabels,
    /// 禁止使用 no unreachable code
    NoNoUnreachableCode,
    /// 禁止使用 no implicit returns
    NoNoImplicitReturns,
    /// 禁止使用 no implicit coercion
    NoNoImplicitCoercion,
    /// 禁止使用 no eval
    NoNoEval,
    /// 禁止使用 no with
    NoNoWith,
    /// 禁止使用 no arguments
    NoNoArguments,
    /// 禁止使用 no delete
    NoNoDelete,
    /// 禁止使用 no void
    NoNoVoid,
    /// 禁止使用 no undefined
    NoNoUndefined,
    /// 禁止使用 no null
    NoNoNull,
    /// 禁止使用 no any
    NoNoAny,
    /// 禁止使用 no never
    NoNoNever,
    /// 禁止使用 no object
    NoNoObject,
    /// 禁止使用 no unknown
    NoNoUnknown,
    /// 禁止使用 no symbol
    NoNoSymbol,
    /// 禁止使用 no bigint
    NoNoBigInt,
    /// 禁止使用 no number
    NoNoNumber,
    /// 禁止使用 no string
    NoNoString,
    /// 禁止使用 no boolean
    NoNoBoolean,
    /// 禁止使用 no array
    NoNoArray,
    /// 禁止使用 no tuple
    NoNoTuple,
    /// 禁止使用 no enum
    NoNoEnum,
    /// 禁止使用 no interface
    NoNoInterface,
    /// 禁止使用 no type
    NoNoType,
    /// 禁止使用 no class
    NoNoClass,
    /// 禁止使用 no function
    NoNoFunction,
    /// 禁止使用 no const
    NoNoConst,
    /// 禁止使用 no let
    NoNoLet,
    /// 禁止使用 no var
    NoNoVar,
    /// 禁止使用 no async
    NoNoAsync,
    /// 禁止使用 no await
    NoNoAwait,
    /// 禁止使用 no yield
    NoNoYield,
    /// 禁止使用 no return
    NoNoReturn,
    /// 禁止使用 no throw
    NoNoThrow,
    /// 禁止使用 no try
    NoNoTry,
    /// 禁止使用 no catch
    NoNoCatch,
    /// 禁止使用 no finally
    NoNoFinally,
    /// 禁止使用 no if
    NoNoIf,
    /// 禁止使用 no else
    NoNoElse,
    /// 禁止使用 no switch
    NoNoSwitch,
    /// 禁止使用 no case
    NoNoCase,
    /// 禁止使用 no default
    NoNoDefault,
    /// 禁止使用 no for
    NoNoFor,
    /// 禁止使用 no while
    NoNoWhile,
    /// 禁止使用 no do
    NoNoDo,
    /// 禁止使用 no break
    NoNoBreak,
    /// 禁止使用 no continue
    NoNoContinue,
    /// 禁止使用 no with statement
    NoNoWithStatement,
    /// 禁止使用 no debugger
    NoNoDebugger,
    /// 禁止使用 no import
    NoNoImport,
    /// 禁止使用 no export
    NoNoExport,
    /// 禁止使用 no from
    NoNoFrom,
    /// 禁止使用 no as
    NoNoAs,
    /// 禁止使用 no in
    NoNoIn,
    /// 禁止使用 no instanceof
    NoNoInstanceOf,
    /// 禁止使用 no new
    NoNoNew,
    /// 禁止使用 no this
    NoNoThis,
    /// 禁止使用 no super
    NoNoSuper,
    /// 禁止使用 no static
    NoNoStatic,
    /// 禁止使用 no public
    NoNoPublic,
    /// 禁止使用 no private
    NoNoPrivate,
    /// 禁止使用 no protected
    NoNoProtected,
    /// 禁止使用 no abstract
    NoNoAbstract,
    /// 禁止使用 no readonly
    NoNoReadonly,
    /// 禁止使用 no optional
    NoNoOptional,
    /// 禁止使用 no rest
    NoNoRest,
    /// 禁止使用 no spread
    NoNoSpread,
    /// 禁止使用 no destructuring
    NoNoDestructuring,
    /// 禁止使用 no arrow functions
    NoNoArrowFunctions,
    /// 禁止使用 no generator functions
    NoNoGeneratorFunctions,
    /// 禁止使用 no async functions
    NoNoAsyncFunctions,
    /// 禁止使用 no method syntax
    NoNoMethodSyntax,
    /// 禁止使用 no computed properties
    NoNoComputedProperties,
    /// 禁止使用 no shorthand properties
    NoNoShorthandProperties,
    /// 禁止使用 no rest properties
    NoNoRestProperties,
    /// 禁止使用 no spread properties
    NoNoSpreadProperties,
    /// 禁止使用 no template literals
    NoNoTemplateLiterals,
    /// 禁止使用 no regex literals
    NoNoRegexLiterals,
    /// 禁止使用 no numeric literals
    NoNoNumericLiterals,
    /// 禁止使用 no string literals
    NoNoStringLiterals,
    /// 禁止使用 no boolean literals
    NoNoBooleanLiterals,
    /// 禁止使用 no null literals
    NoNoNullLiterals,
    /// 禁止使用 no undefined literals
    NoNoUndefinedLiterals,
    /// 禁止使用 no bigint literals
    NoNoBigIntLiterals,
    /// 禁止使用 no symbol literals
    NoNoSymbolLiterals,
    /// 禁止使用 no object literals
    NoNoObjectLiterals,
    /// 禁止使用 no array literals
    NoNoArrayLiterals,
    /// 禁止使用 no function literals
    NoNoFunctionLiterals,
    /// 禁止使用 no class literals
    NoNoClassLiterals,
    /// 禁止使用 no namespace literals
    NoNoNamespaceLiterals,
    /// 禁止使用 no module literals
    NoNoModuleLiterals,
    /// 禁止使用 no enum literals
    NoNoEnumLiterals,
    /// 禁止使用 no type literals
    NoNoTypeLiterals,
    /// 禁止使用 no interface literals
    NoNoInterfaceLiterals,
    /// 禁止使用 no union literals
    NoNoUnionLiterals,
    /// 禁止使用 no intersection literals
    NoNoIntersectionLiterals,
    /// 禁止使用 no literal types
    NoNoLiteralTypes,
    /// 禁止使用 no tuple types
    NoNoTupleTypes,
    /// 禁止使用 no function types
    NoNoFunctionTypes,
    /// 禁止使用 no object types
    NoNoObjectTypes,
    /// 禁止使用 no array types
    NoNoArrayTypes,
    /// 禁止使用 no indexed access types
    NoNoIndexedAccessTypes,
    /// 禁止使用 no keyof types
    NoNoKeyofTypes,
    /// 禁止使用 no typeof types
    NoNoTypeofTypes,
    /// 禁止使用 no inferred types
    NoNoInferredTypes,
    /// 禁止使用 no recursive types
    NoNoRecursiveTypes,
    /// 禁止使用 no generic constraints
    NoNoGenericConstraints,
    /// 禁止使用 no type parameter defaults
    NoNoTypeParameterDefaults,
    /// 禁止使用 no rest type parameters
    NoNoRestTypeParameters,
    /// 禁止使用 no optional type parameters
    NoNoOptionalTypeParameters,
    /// 禁止使用 no overloaded function types
    NoNoOverloadedFunctionTypes,
    /// 禁止使用 no constructor types
    NoNoConstructorTypes,
    /// 禁止使用 no access modifiers
    NoNoAccessModifiers,
    /// 禁止使用 no static members
    NoNoStaticMembers,
    /// 禁止使用 no abstract classes and methods
    NoNoAbstractClasses,
    /// 禁止使用 no inheritance
    NoNoInheritance,
    /// 禁止使用 no interface implementation
    NoNoInterfaceImplementation,
    /// 禁止使用 no module system
    NoNoModuleSystem,
    /// 禁止使用 no ES modules
    NoNoEsModules,
    /// 禁止使用 no CommonJS modules
    NoNoCommonJsModules,
    /// 禁止使用 no external module declarations
    NoNoExternalModuleDeclarations,
    /// 禁止使用 no triple-slash directives
    NoNoTripleSlashDirectives,
    /// 禁止使用 no JSDoc comments
    NoNoJsDocComments,
}
