use crate::environment::Environment;
use crate::expr::Expr;
use crate::stmt::Stmt;
use crate::token::{Literal, Token, TokenType};


#[derive(Clone)]
pub struct NativeFunction {
    pub name: String,
    pub arity: usize,
    pub callable: fn(&mut Interpreter, Vec<Value>) -> Result<Value, RuntimeError>,
}

impl std::fmt::Debug for NativeFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "NativeFunction({})", self.name)
    }
}

trait Callable {
    fn arity(&self, interpreter: &Interpreter) -> usize;
    fn call(&self, interpreter: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError>;
}

impl Callable for NativeFunction {
    fn arity(&self, _interpreter: &Interpreter) -> usize {
        self.arity
    }
    fn call(&self, interpreter: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
        (self.callable)(interpreter, args)
    }
}

#[derive(Clone)]
pub struct LoxFunction {
    name: String,
    body: Vec<Stmt>,
    params: Vec<Token>,
    arity: usize,
}

impl Callable for LoxFunction {
    fn arity(&self, _interpreter: &Interpreter) -> usize {
        self.arity
    }

    fn call(&self, interpreter: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
        let mut environment = Environment::default();
        for (i, arg) in args.iter().enumerate() {
            environment.define(self.params[i].lexeme.clone(), Some(arg.clone()));
        }

        // Note: Not modifying outer variables from inside of function
        interpreter.interpret_block(self.body.clone(), Some(Box::new(environment.clone())));

        Ok(Value::Null)
    }
}


pub enum Value {
    Bool(bool),
    Null,
    Number(f64),
    String(String),
    NativeFunction(NativeFunction),
    LoxFunction(LoxFunction)
}

impl Clone for Value {
    fn clone(&self) -> Value {
        match self {
            Value::Bool(b) => Value::Bool(*b),
            Value::Null => Value::Null,
            Value::Number(n) => Value::Number(*n),
            Value::String(s) => Value::String(s.clone()),
            Value::LoxFunction(f) => Value::LoxFunction((*f).clone()),
            Value::NativeFunction(f) => Value::NativeFunction((*f).clone()),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Value::Number(n) => f.write_str(format!("{}", n).as_str()),
            Value::String(s) => f.write_str(s.as_str()),
            Value::Null => f.write_str("Null"),
            Value::Bool(b) => f.write_str(b.to_string().as_str()),
            Value::LoxFunction(fu) => f.write_str(fu.name.as_str()),
            Value::NativeFunction(fu) => f.write_str(fu.name.as_str()),
        }
    }
}

#[derive(Debug)]
pub enum RuntimeError {
    BinaryOperationError,
    // NotImplementedError,
    VariableNotFound,
    VariableNotInitialized,
    LogicalOperatorError,
    InvalidCall(String),
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RuntimeError::BinaryOperationError => f.write_str("BinaryOperationError"),
            // InterpreterError::NotImplementedError => f.write_str("NotImplementedError"),
            RuntimeError::VariableNotFound => f.write_str("VariableNotFound"),
            RuntimeError::VariableNotInitialized => f.write_str("VariableNotInitialized"),
            RuntimeError::LogicalOperatorError => f.write_str("LogicalOperatorError"),
            RuntimeError::InvalidCall(m) => f.write_str(format!("InvalidCall: {}", m).as_str()),
        }
    }
}

#[derive(Default)]
pub struct Interpreter {
    environment: Box<Environment>,
}

impl Interpreter {
    pub fn interpret(&mut self, statements: Vec<Stmt>) {
        self.environment = Box::new(Environment::default());
        self.environment.enclosing(None);
        for statement in statements {
            self.execute(statement);
        }
    }

    fn execute(&mut self, statement: Stmt) {
        match statement {
            Stmt::Print(s) => println!("{}", self.interpret_expr(Box::new(s)).expect("Failed to interpret")),
            Stmt::Block(b) => self.interpret_block(b, None),
            Stmt::Expression(e) => {
                let res = self.interpret_expr(Box::new(e)).expect("Failed to interpret");
                println!("{}", res)
            },
            Stmt::VarDeclaration(n, e) => {
                let val = match e {
                    Some(ex) =>  Some(self.interpret_expr(Box::new(ex))
                        .expect("Failed to interpret variable declaration")),
                    None => None,
                };
                self.environment.define(n.lexeme, val);
            }
            Stmt::If(c, b1, b2) => {
                let condition = match self.interpret_expr(Box::new(c)) {
                    Ok(c) => c,
                    Err(e) => {eprintln!("Failed interpreting condition:  {}", e); return;}
                };
                if self.is_truthy(condition) {
                    self.execute(*b1);
                } else if b2.is_some() {
                    self.execute(*b2.unwrap());
                }
            }
            Stmt::While(condition, body) => {
                loop {
                    if let Ok(result) = self.interpret_expr(Box::new(condition.clone())) {
                        if self.is_truthy(result) {
                            self.execute(*body.clone());
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
            Stmt::Function(name, ref arguments, body) => {
                let func = LoxFunction{name: name.lexeme.clone(), body, params: arguments.clone(), arity: arguments.len()};
                self.environment.define(name.lexeme, Some(Value::LoxFunction(func)));
            }
        }
    }

    fn interpret_block(&mut self, block: Vec<Stmt>, environment: Option<Box<Environment>>) {
        let prev_env = self.environment.clone();
        if let Some(e) = environment {
            self.environment = e;
        } else {
            self.environment = Box::new(Environment::default());
            self.environment.enclosing(Some(prev_env));
        }
        for stmt in block {
            self.execute(stmt)
        }
        if let Some(enclosing) = self.environment.get_enclosing() {
            self.environment = enclosing;
        }
    }

    // TODO is it better to use non-boxed expr argument?
    fn interpret_expr(&mut self, expr: Box<Expr>) -> Result<Value, RuntimeError> {
        match *expr {
            Expr::Literal(literal) => self.interpret_literal(literal),
            Expr::Unary(op, e) => self.interpret_unary(op.token_type, e),
            Expr::Binary(left, operator, right) =>
                self.interpret_binary(left, operator.token_type, right),
            Expr::Grouping(e) => self.interpret_expr(e),
            Expr::Variable(v) => self.environment.get(v.lexeme),
            Expr::Assign(literal, e) => {
                let res = self.interpret_expr(e)?;
                self.environment.assign(literal.lexeme, res.clone())?;
                Ok(res)
            },
            Expr::Logical(left, operator, right) =>
                self.interpret_logical(left, operator.token_type, right),
            Expr::Call(callee, _paren, arguments) => {
                let callee = self.interpret_expr(callee)?;
                let mut arguments_ = vec![];
                for argument in arguments {
                    arguments_.push(self.interpret_expr(Box::new(argument))?);
                }

                if let Value::LoxFunction(function) = callee {
                    function.call(self, arguments_)
                } else {
                    Err(RuntimeError::InvalidCall(String::from("Expected function call")))
                }
            }
            // _ => Err(InterpreterError::NotImplementedError),
        }
    }

    fn interpret_literal(&self, literal: Literal) -> Result<Value, RuntimeError> {
        return match literal {
            Literal::False => Ok(Value::Bool(false)),
            Literal::True => Ok(Value::Bool(true)),
            Literal::Null => Ok(Value::Null),
            Literal::String(s) => Ok(Value::String(s.clone())),
            Literal::Number(n) => Ok(Value::Number(n)),
            Literal::Identifier(n) => self.environment.get(n),
            // _ => Err(InterpreterError::NotImplementedError),
        }
    }

    fn interpret_unary(&mut self, operator: TokenType, expr: Box<Expr>) -> Result<Value, RuntimeError> {
        let right = self.interpret_expr(expr)?;
        return match (operator, &right) {
            (TokenType::Minus, Value::Number(n)) => Ok(Value::Number(-1.0 * n)),
            (TokenType::Bang, _) => Ok(Value::Bool(!self.is_truthy(right))),
            _ => panic!("Unary Not implemented.")
        }
    }

    fn interpret_logical(&mut self, left: Box<Expr>, operator: TokenType, right: Box<Expr>) -> Result<Value, RuntimeError> {
        let left = self.interpret_expr(left)?;

        match operator {
            TokenType::Or => {
                if self.is_truthy(left.clone()) {
                    return Ok(left);
                }
            },
            TokenType::And => {
                if !self.is_truthy(left.clone()) {
                    return Ok(left);
                }
            },
            _ => return Err(RuntimeError::LogicalOperatorError)
        }

        self.interpret_expr(right)
    }

    fn interpret_binary(&mut self, left: Box<Expr>, operator: TokenType, right: Box<Expr>) -> Result<Value, RuntimeError> {
        let left = self.interpret_expr(left)?;
        let right = self.interpret_expr(right)?;

        return match (left, operator, right) {
            (Value::Number(n1), TokenType::Minus, Value::Number(n2)) => Ok(Value::Number(n1 - n2)),
            (Value::Number(n1), TokenType::Plus, Value::Number(n2)) => Ok(Value::Number(n1 + n2)),
            (Value::String(s1), TokenType::Plus, Value::String(s2)) => Ok(Value::String([s1, s2].join(""))),
            (Value::Number(n1), TokenType::Slash, Value::Number(n2))  => Ok(Value::Number(n1 / n2)),
            (Value::Number(n1), TokenType::Star, Value::Number(n2))  => Ok(Value::Number(n1 * n2)),
            (Value::Number(n1), TokenType::Greater, Value::Number(n2))  => Ok(Value::Bool(n1 > n2)),
            (Value::Number(n1), TokenType::GreaterEqual, Value::Number(n2))  => Ok(Value::Bool(n1 >= n2)),
            (Value::Number(n1), TokenType::Less, Value::Number(n2))  => Ok(Value::Bool(n1 < n2)),
            (Value::Number(n1), TokenType::LessEqual, Value::Number(n2))  => Ok(Value::Bool(n1 <= n2)),
            (Value::Number(n1), TokenType::BangEqual, Value::Number(n2))  => Ok(Value::Bool(n1 != n2)),
            (Value::Number(n1), TokenType::EqualEqual, Value::Number(n2))  => Ok(Value::Bool(n1 == n2)),
            _ => Err(RuntimeError::BinaryOperationError),
        }
    }

    fn is_truthy(&self, value: Value) -> bool {
        return match value {
            Value::Bool(b) => b,
            Value::Number(n) => n != 0.0,
            Value::String(s) => s.len() != 0,
            Value::Null => false,
            Value::NativeFunction(_nf) => false,
            Value::LoxFunction(_lf) => false,
        }
    }
}