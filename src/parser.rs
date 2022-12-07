use core::panic;
use std::collections::{VecDeque};

use crate::lexer::{LexerToken};
use crate::value::Value;
use crate::expr::{AstExpr};

pub struct Parser {
    input: VecDeque<LexerToken>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParserToken {
    DeclareVariable(String), // Pops a value from stack and stores it to stack
    DeclareFunction(String, Vec<ParserToken>),
    GetVariable(String),     // Pushes the variables value to stack
    Operation(String),       // Pops 2 values from stack as arguments and pushes a result
    Push(Value),
    Pop(),
    Call(String, u8),        // Second argument for amount of arguments
    Ret(),
}

impl Parser {
    pub fn parse(tokens: VecDeque<LexerToken>) -> Vec<ParserToken> {
        let mut parser = Parser::new(tokens);
        parser.parse_until(LexerToken::Eof())
    }

    /**
     * Doesn't not append to tokens, like parse does.
     */
    fn parse_until(&mut self, tk: LexerToken) -> Vec<ParserToken> {
        let mut tokens = vec![];
        'parse_loop : loop {
            let token = self.peek().expect("Unexpted end of file");
            if token == &tk {
                break 'parse_loop;
            }
            
            if let LexerToken::Keyword(kw) = token {
                match kw.as_str() {
                    "let" => { 
                        tokens.append(&mut self.variable_decleration());
                    },
                    "fn" => { 
                        tokens.append(&mut self.function_decleration());
                    },
                    _ => { panic!("Unimplumented keyword {}", kw); }
                }
            }
            else if let LexerToken::Identifier(fn_name) = token.clone() {
                self.eat(); // Identifier
                let next = self.eat().expect("Syntax error");
                if LexerToken::Operator("(".to_string()) == next {
                    tokens.append(&mut self.function_call(fn_name.clone()));
                }
            }
            else 
            {
                let mut expr = self.eat_expr(vec![LexerToken::Symbol(';')]);
                return AstExpr::evaluate(&mut expr);
            }
        }
        tokens
    }

    #[must_use]
    fn variable_decleration(&mut self) -> Vec<ParserToken> {
        let mut tokens = vec![];

        // eat "let" keyword
        self.eat();

        // identifier
        let tk_identifier = self.eat().expect("expected an identifier after 'let' keyword");
        if let LexerToken::Identifier(identifier) = tk_identifier {
            // symbol '='
            self.eat_expect(LexerToken::Symbol('='));
            
            // Get expression
            let mut expr = self.eat_expr(vec![LexerToken::Symbol(';')]);
            let mut evaluated = AstExpr::evaluate(&mut expr);

            // symbol ';'
            self.eat_expect(LexerToken::Symbol(';'));

            // push ParserTokens
            tokens.append(&mut evaluated);
            tokens.push(ParserToken::DeclareVariable(identifier.clone()));
            return tokens;
        }
        panic!("expected an identifier after 'let' keyword");
    }

    #[must_use]
    fn function_decleration(&mut self) -> Vec<ParserToken> {
        // eat "fn" keyword
        self.eat();

        // identifier
        let tk_identifier = self.eat().expect("expected an identifier after 'fn' keyword");
        if let LexerToken::Identifier(fn_name) = tk_identifier {
            // eat operator '('
            self.eat_expect(LexerToken::Operator("(".to_string()));

            // get argument names
            let mut fn_args: Vec<String> = vec![];
            'args : loop {
                let tk = self.eat().expect("Invalid function decleration");

                if let LexerToken::Identifier(arg_identifier) = tk {
                    fn_args.push(arg_identifier);
                    let peek = self.eat().expect("Invalid function decleration");
                    if peek == LexerToken::Symbol(',') {
                        continue;
                    }
                    else if peek == LexerToken::Operator(")".to_string()) {
                        break 'args;
                    }
                    panic!("Syntax error");
                }
                else if tk == LexerToken::Operator(")".to_string()) {
                    break 'args;
                }
                else {
                    panic!("Syntax error");
                }
            }
            self.eat_expect(LexerToken::Symbol('{'));

            let fn_body = self.parse_until(LexerToken::Symbol('}'));
            self.eat_expect(LexerToken::Symbol('}'));

            // push tokens
            return vec![(ParserToken::DeclareFunction(fn_name, fn_body))];
        }
        panic!("expected an identifier after 'fn' keyword");
    }

    #[must_use]
    fn function_call(&mut self, fn_name: String) -> Vec<ParserToken> {
        // TODO: Parse Arguments
        self.eat_expect(LexerToken::Operator(")".to_string()));
        self.eat_expect(LexerToken::Symbol(';'));
        return vec![ParserToken::Call(fn_name, 0)];
    }

    /**
     * Terminator is used to determine when the expression is suppost to end, terminator doesn't get eaten. e.g: 
     * "LexerToken::Symbol(';')" for "let x = 2+2;"
     * "LexerToken::Symbol(',')" for "fn foo(2+2+2, 0)"
     * "LexerToken::Operator(')')" for "fn foo(2+2+2)" // this is going to be a fucking problem, lol.
     */
    fn eat_expr(&mut self, terminator: Vec<LexerToken>) -> VecDeque<LexerToken> {
        let mut out_tks = VecDeque::new();
        'get_tokens: loop {
            let peeked = self.peek();
            if peeked.is_none() {
                panic!("Expected '{:?}' got EOF instead!", terminator);
            }
            if terminator.contains(peeked.unwrap()) {
                break 'get_tokens;
            }

            let token = self.eat().unwrap();
            out_tks.push_back(token);
        }
        return out_tks;
    }

    fn peek(&self) -> Option<&LexerToken> {
        if self.input.len() == 0 {
            return None;
        }
        self.input.front()
    }

    fn eat_checked(&mut self) -> LexerToken {
        let popped = self.eat();
        if popped.is_none() {
            panic!("Got unexpected EOF");
        }
        popped.unwrap()
    }

    fn eat_expect(&mut self, expect: LexerToken) -> LexerToken {
        let popped = self.eat();
        if popped.is_none() {
            panic!("Expected {:?} got EOF instead!", expect);
        }
        let tk = popped.unwrap();
        if tk != expect {
            panic!("Expected {:?} got {:?} instead! :(", expect, tk);
        }
        tk
    }

    fn eat(&mut self) -> Option<LexerToken> {
        self.input.pop_front()
    }

    fn new(tks: VecDeque<LexerToken>) -> Parser { 
        Parser {
            input: tks,
        }
    }
}
