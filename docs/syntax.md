# Syntax

### Program

```text
program -> statement* EOF
```

### Statements

```text
statement            -> (declarationStatement
                      | expressionStatement) ';'

expressionStatement  -> expression

declarationStatement -> classDeclaration
                      | functionDeclaration
                      | variableDeclaration

classDeclaration     -> 'class' IDENTIFIER (':' IDENTIFIER)? '{' functionDeclaration* '}'

functionDeclaration  -> 'fn' IDENTIFIER '(' IDENTIFIER* ')' blockExpression
                      | => expression

variableDeclaration  -> 'let' IDENTIFIER '=' expression ';'
```

### Expressions

```text
closure              -> '|' IDENTIFIER* '|' '=>' expression

expression          -> assignment | controlFlowExpression

assignment          -> IDENTIFIER (('.' IDENTIFIER)*)? '=' logic_or

logic_or            -> logic_and ('or' logic_and)*

logic_and           -> equality  ('and' equality)*

equality            -> comparison (( '!=' | '==' ) comparison )*

comparison          -> term (( '>' | '>=' | '<' | '<=') term)*

term                -> factor (( '-' | '+' ) factor)*

factor              -> unary (( '/' | '*' ) unary)*

unary               -> ('!' | '-') unary | call

call                -> primary ( '(' arguments? ')' )*

primary             -> boolean | number | array 
                     | string | "self" | identifier ('.' identifier)*
```

### Control flow expression

```text
if                  -> 'if' expression block ('else if' block)* ('else' block)?

while               -> 'while' expression block

return              -> 'return' expression?

block               -> '{' statement* expression? '}'

forIn               -> 'for' identifier 'in' expression block

break               -> 'break' expression?

continue            -> 'continue'

```

### Literals

```text
array               -> '[' primary* ']'   

identifier          -> alpha (alpha | digit)*
                     
boolean             -> 'true' | 'false'

number              -> '.'? digit+ '.'? digit* 

digit               -> '0' .. '9'

alpha               -> 'a' ... 'z' | 'A' ... 'Z' | '_' 

string              -> '"' <any char except '"'>* '"'
```