use crossterm::style::Color;

use crate::parser::{
    ast::{
        expr::{
            argument::{Argument, ArgumentPartKind, Expand, ExpandKind},
            closure::Closure,
            command::{CommandPart, CommandPartKind},
            Expr, ExprKind,
        },
        literal::{Literal, LiteralKind},
        statement::{Statement, StatementKind},
        variable::Variable,
        Ast, Block, Compound,
    },
    lexer::token::span::{Span, Spanned},
};

#[derive(Debug, Clone, Copy)]
pub enum ColorType {
    Base,
    String,
    Literal,
    Command,
    Keyword,
    Variable,
    Operator,
    Flag,
    Regex,
    FunctionName,
}

impl ColorType {
    pub fn to_color(self) -> Color {
        match self {
            ColorType::Base => Color::Grey,
            ColorType::String => Color::Green,
            ColorType::Literal => Color::Yellow,
            ColorType::Command => Color::Cyan,
            ColorType::Keyword => Color::Magenta,
            ColorType::Variable => Color::Red,
            ColorType::Operator => Color::Magenta,
            ColorType::Flag => Color::Blue,
            ColorType::Regex => Color::Blue,
            ColorType::FunctionName => Color::Blue,
        }
    }
}

#[derive(Default, Debug)]
pub struct HighlightVisitor {
    pub spans: Vec<Spanned<ColorType>>,
}

impl HighlightVisitor {
    pub fn visit_ast(&mut self, ast: &Ast) {
        for compound in &ast.sequence {
            self.visit_compound(compound);
        }
    }

    pub fn visit_compound(&mut self, compound: &Compound) {
        match compound {
            Compound::Statement(statement) => self.visit_statement(statement),
            Compound::Expr(expr) => self.visit_expr(expr),
        }
    }

    pub fn visit_expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Call(cmd, arguments) => {
                // TODO highlight &
                self.visit_command(cmd);
                self.visit_arguments(arguments);
            }
            ExprKind::Pipe(pipe) => {
                // TODO highlight pipe
                for expr in pipe {
                    self.visit_expr(expr);
                }
            }
            ExprKind::Redirection { arg, .. } => {
                // TODO highlight >
                self.visit_argument(arg);
            }
            ExprKind::Variable(variable) => self.visit_variable(variable),
            ExprKind::Binary(op, lhs, rhs) => {
                self.visit_expr(lhs);
                self.spans.push(Spanned::new(ColorType::Operator, op.span));
                self.visit_expr(rhs);
            }
            ExprKind::Unary(op, expr) => {
                self.spans.push(Spanned::new(ColorType::Operator, op.span));
                self.visit_expr(expr);
            }
            ExprKind::Literal(literal) => self.visit_literal(literal),
            ExprKind::SubExpr(expr) => self.visit_expr(expr),
            // TODO add color to column
            ExprKind::Column(expr, _) => {
                self.visit_expr(expr);
            }
            ExprKind::ErrorCheck(inner_expr) => {
                self.spans.push(Spanned::new(
                    ColorType::Operator,
                    Span::new(expr.span.start(), expr.span.start() + 1),
                ));
                self.visit_expr(inner_expr);
            }
            ExprKind::Index { expr, index } => {
                self.visit_expr(expr);
                self.visit_expr(index);
            }
            ExprKind::Closure(closure) => self.visit_closure(closure),
        }
    }

    pub fn visit_closure(&mut self, closure: &Closure) {
        let Closure {
            arg_span,
            parameters,
            block,
            ..
        } = closure;
        self.spans.push(Spanned::new(
            ColorType::Operator,
            Span::new(arg_span.start(), arg_span.start() + 1),
        ));
        for variable in parameters {
            self.visit_variable(variable);
        }
        self.spans.push(Spanned::new(
            ColorType::Operator,
            Span::new(arg_span.end() - 1, arg_span.end()),
        ));
        self.visit_block(block);
    }

    pub fn visit_arguments(&mut self, arguments: &[Argument]) {
        for argument in arguments {
            self.visit_argument(argument);
        }
    }

    pub fn visit_argument(&mut self, argument: &Argument) {
        for part in &argument.parts {
            match &part.kind {
                ArgumentPartKind::Variable(variable) => self.visit_variable(variable),
                ArgumentPartKind::Expand(expand) => self.visit_expand(expand),
                ArgumentPartKind::Bare(string) => {
                    if string.starts_with('-') {
                        self.spans.push(Spanned::new(ColorType::Flag, part.span))
                    } else {
                        self.spans.push(Spanned::new(ColorType::String, part.span))
                    }
                }
                ArgumentPartKind::Float(_) => {
                    self.spans.push(Spanned::new(ColorType::Literal, part.span))
                }
                ArgumentPartKind::Int(_) => {
                    self.spans.push(Spanned::new(ColorType::Literal, part.span))
                }
                ArgumentPartKind::Quoted(_) => {
                    self.spans.push(Spanned::new(ColorType::String, part.span))
                }
                ArgumentPartKind::Expr(expr) => self.visit_expr(expr),
            }
        }
    }

    pub fn visit_command(&mut self, cmd: &[CommandPart]) {
        // TODO add check if command exists
        for part in cmd {
            match &part.kind {
                CommandPartKind::Expand(expand) => self.visit_expand(expand),
                CommandPartKind::String(_) => {
                    self.spans.push(Spanned::new(ColorType::Command, part.span))
                }
                CommandPartKind::Variable(variable) => self.visit_variable(variable),
            }
        }
    }

    pub fn visit_literal(&mut self, literal: &Literal) {
        match &literal.kind {
            LiteralKind::String(_) => self
                .spans
                .push(Spanned::new(ColorType::String, literal.span)),
            LiteralKind::Expand(expand) => self.visit_expand(expand),
            LiteralKind::List(list) => {
                for expr in list {
                    self.visit_expr(expr);
                }
            }
            LiteralKind::Map(map) => {
                for (key, value) in map {
                    self.visit_expr(key);
                    self.visit_expr(value);
                }
            }
            LiteralKind::Float(_) => self
                .spans
                .push(Spanned::new(ColorType::Literal, literal.span)),
            LiteralKind::Int(_) => self
                .spans
                .push(Spanned::new(ColorType::Literal, literal.span)),
            LiteralKind::Bool(_) => self
                .spans
                .push(Spanned::new(ColorType::Literal, literal.span)),
            LiteralKind::Regex(_) => self
                .spans
                .push(Spanned::new(ColorType::Regex, literal.span)),
        }
    }

    pub fn visit_expand(&mut self, expand: &Expand) {
        let mut index = expand.span.start();
        self.spans
            .push(Spanned::new(ColorType::String, Span::new(index, index + 1)));
        index += 1;
        for kind in &expand.content {
            match kind {
                ExpandKind::String(string) => {
                    self.spans.push(Spanned::new(
                        ColorType::String,
                        Span::new(index, index + string.len()),
                    ));
                    index += string.len()
                }
                ExpandKind::Expr(expr) => {
                    self.visit_expr(expr);
                    index = expr.span.end();
                }
                ExpandKind::Variable(variable) => {
                    self.visit_variable(variable);
                    index = variable.span.end();
                }
            }
        }
        self.spans
            .push(Spanned::new(ColorType::String, Span::new(index, index + 1)));
    }

    pub fn visit_variable(&mut self, variable: &Variable) {
        // TODO add check if variable exists
        // TODO fix variable span
        self.spans
            .push(Spanned::new(ColorType::Variable, variable.span));
    }

    pub fn visit_statement(&mut self, statement: &Statement) {
        match &statement.kind {
            StatementKind::Export(variable, expr) => {
                let start = statement.span.start();
                self.spans.push(Spanned::new(
                    ColorType::Keyword,
                    Span::new(start, start + 6),
                ));
                self.visit_variable(variable);
                self.spans.push(Spanned::new(
                    ColorType::Keyword,
                    Span::new(variable.span.end(), expr.span.start()),
                ));
                self.visit_expr(expr);
            }
            StatementKind::Declaration(variable, expr) => {
                let start = statement.span.start();
                self.spans.push(Spanned::new(
                    ColorType::Keyword,
                    Span::new(start, start + 3),
                ));
                self.visit_variable(variable);
                self.spans.push(Spanned::new(
                    ColorType::Keyword,
                    Span::new(variable.span.end(), expr.span.start()),
                ));
                self.visit_expr(expr);
            }
            StatementKind::Assign(variable, expr) => {
                self.visit_variable(variable);
                self.spans.push(Spanned::new(
                    ColorType::Keyword,
                    Span::new(variable.span.end(), expr.span.start()),
                ));
                self.visit_expr(expr);
            }
            StatementKind::AssignOp(variable, op, expr) => {
                self.visit_variable(variable);
                self.spans.push(Spanned::new(
                    ColorType::Keyword,
                    Span::new(op.span.start(), op.span.end()),
                ));
                self.visit_expr(expr);
            }
            StatementKind::If(expr, block, next) => {
                self.spans.push(Spanned::new(
                    ColorType::Keyword,
                    Span::new(statement.span.start(), statement.span.start() + 2),
                ));
                self.visit_expr(expr);
                self.visit_block(block);
                if let Some(next) = next {
                    self.spans.push(Spanned::new(
                        ColorType::Keyword,
                        Span::new(block.span.end(), next.span.start()),
                    ));
                    self.visit_statement(next);
                }
            }
            StatementKind::Fn(_, function) => {
                self.spans.push(Spanned::new(
                    ColorType::Keyword,
                    Span::new(statement.span.start(), statement.span.start() + 2),
                ));
                self.spans.push(Spanned::new(
                    ColorType::FunctionName,
                    Span::new(statement.span.start() + 2, function.arg_span.start()),
                ));
                for variable in &function.parameters {
                    self.visit_variable(variable);
                }
                self.visit_block(&function.block);
            }
            StatementKind::Return(expr) => {
                self.spans.push(Spanned::new(
                    ColorType::Keyword,
                    Span::new(statement.span.start(), statement.span.start() + 6),
                ));
                if let Some(expr) = expr {
                    self.visit_expr(expr);
                }
            }
            StatementKind::For(variable, expr, block) => {
                self.spans.push(Spanned::new(
                    ColorType::Keyword,
                    Span::new(statement.span.start(), statement.span.start() + 3),
                ));
                self.visit_variable(variable);
                self.spans.push(Spanned::new(
                    ColorType::Keyword,
                    Span::new(variable.span.end(), expr.span.start()),
                ));
                self.visit_expr(expr);
                self.visit_block(block);
            }
            StatementKind::While(expr, block) => {
                self.spans.push(Spanned::new(
                    ColorType::Keyword,
                    Span::new(statement.span.start(), statement.span.start() + 5),
                ));
                self.visit_expr(expr);
                self.visit_block(block);
            }
            StatementKind::Loop(block) => {
                self.spans.push(Spanned::new(
                    ColorType::Keyword,
                    Span::new(statement.span.start(), statement.span.start() + 6),
                ));
                self.visit_block(block);
            }
            StatementKind::TryCatch(block, catch) => {
                self.spans.push(Spanned::new(
                    ColorType::Keyword,
                    Span::new(statement.span.start(), statement.span.start() + 3),
                ));
                self.visit_block(block);
                self.spans.push(Spanned::new(
                    ColorType::Keyword,
                    Span::new(block.span.end(), catch.span.start()),
                ));
                self.visit_block(catch);
            }
            StatementKind::Block(block) => self.visit_block(block),
            StatementKind::Continue => self.spans.push(Spanned::new(
                ColorType::Keyword,
                Span::new(statement.span.start(), statement.span.start() + 8),
            )),
            StatementKind::Break => self.spans.push(Spanned::new(
                ColorType::Keyword,
                Span::new(statement.span.start(), statement.span.start() + 5),
            )),
        }
    }

    pub fn visit_block(&mut self, block: &Block) {
        for compound in &block.sequence {
            self.visit_compound(compound);
        }
    }
}
